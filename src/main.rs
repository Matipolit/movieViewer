use iced::widget::{button, column, container, row, text};
use iced::{keyboard, subscription, window};
use iced::{Alignment, Application, Command, Event, Length, Settings, Subscription};

use native_dialog::FileDialog;
use walkdir::WalkDir;

use self::theme::Theme;
use self::widget::Element;

pub mod config;

fn main() -> iced::Result {
    MovieViewer::run(Settings::default())
}

//theming

mod widget {
    #![allow(dead_code)]
    use crate::theme::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
}

mod theme {
    use iced::widget::{button, container, text};
    use iced::{application, color, Background, BorderRadius, Color};

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Theme;

    impl application::StyleSheet for Theme {
        type Style = ();

        fn appearance(&self, _style: &Self::Style) -> application::Appearance {
            application::Appearance {
                background_color: Color::WHITE,
                text_color: Color::BLACK,
            }
        }
    }

    impl text::StyleSheet for Theme {
        type Style = ();

        fn appearance(&self, _style: Self::Style) -> text::Appearance {
            text::Appearance {
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub enum Container {
        #[default]
        Default,
        Bordered,
    }

    impl container::StyleSheet for Theme {
        type Style = Container;

        fn appearance(&self, style: &Self::Style) -> container::Appearance {
            match style {
                Container::Default => container::Appearance::default(),
                Container::Bordered => container::Appearance {
                    border_color: color!(0xcc, 0xcc, 0xcc),
                    border_width: 2.0,
                    border_radius: BorderRadius::from(6.0),
                    background: Some(Background::Color(color!(0xee, 0xee, 0xee))),
                    ..Default::default()
                },
            }
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub enum Button {
        #[default]
        Primary,
        Secondary,
    }

    impl button::StyleSheet for Theme {
        type Style = Button;

        fn active(&self, style: &Self::Style) -> button::Appearance {
            match style {
                Button::Primary => button::Appearance {
                    border_radius: BorderRadius::from(4.0),
                    border_width: 1.0,
                    border_color: color!(0xcc, 0xcc, 0xcc),
                    ..Default::default()
                },
                Button::Secondary => button::Appearance {
                    ..Default::default()
                },
            }
        }
    }
}
//data

#[derive(Debug, Clone)]
struct MovieFolder {
    path: String,
    movies: Vec<Movie>,
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    SystemError,
}

#[derive(Debug, Clone)]
enum MovieRating {
    G,
    PG,
    PG13,
    R,
    NC17,
    Unrated,
}

impl MovieRating {
    fn as_str(&self) -> &'static str {
        match self {
            MovieRating::G => "G",
            MovieRating::PG => "PG",
            MovieRating::PG13 => "PG-13",
            MovieRating::R => "R",
            MovieRating::NC17 => "NC-17",
            MovieRating::Unrated => "Unrated",
        }
    }
}

#[derive(Debug, Clone)]
struct Movie {
    path: String,
    title: String,
    year: u32,
    rated: MovieRating,
}

#[derive(Debug)]
enum MovieViewer {
    Selecting,
    Loading { path: String },
    Loaded { folder: MovieFolder },
    Errored,
}

impl MovieFolder {
    async fn select() -> Result<String, Error> {
        let path = FileDialog::new()
            .set_location("~/")
            .show_open_single_dir()
            .unwrap();

        match path {
            Some(path) => return Ok(path.to_str().unwrap().to_string()),
            None => return Err(Error::SystemError),
        };
    }
    async fn load(path: String) -> Result<MovieFolder, Error> {
        let mut movies: Vec<Movie> = vec![];
        for entry in WalkDir::new(&path)
            .follow_links(true)
            .max_depth(1)
            .min_depth(1)
        {
            let entry_unwr = entry.unwrap();
            let entry_path = entry_unwr.path();
            if entry_path.is_dir() {
                movies.push(Movie {
                    path: entry_path.to_str().unwrap().to_string(),
                    title: entry_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    year: 2002,
                    rated: MovieRating::R,
                });
            }
        }
        Ok(MovieFolder { path, movies })
    }
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    FolderSelect,
    FolderSelected(Result<String, Error>),
    FolderLoaded(Result<MovieFolder, Error>),
}

impl Application for MovieViewer {
    type Theme = Theme;
    type Message = Message;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (MovieViewer, Command<Message>) {
        (
            MovieViewer::Selecting,
            //Command::none(),
            iced::window::change_mode(iced::window::Mode::Fullscreen),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events().map(Message::EventOccurred)
    }

    fn title(&self) -> String {
        "MovieViewer".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                }) = event
                {
                    if key_code == keyboard::KeyCode::Escape {
                        window::close::<Message>()
                    } else if key_code == keyboard::KeyCode::Enter {
                        match self {
                            MovieViewer::Selecting => {
                                Command::perform(MovieFolder::select(), Message::FolderSelected)
                            }
                            _ => Command::none(),
                        }
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            Message::FolderSelect => {
                Command::perform(MovieFolder::select(), Message::FolderSelected)
            }
            Message::FolderSelected(Err(_error)) => {
                *self = MovieViewer::Errored;
                Command::none()
            }
            Message::FolderSelected(Ok(path)) => {
                *self = MovieViewer::Loading { path: path.clone() };
                Command::perform(MovieFolder::load(path), Message::FolderLoaded)
            }
            Message::FolderLoaded(Ok(folder)) => {
                *self = MovieViewer::Loaded { folder };
                Command::none()
            }
            Message::FolderLoaded(Err(_error)) => {
                *self = MovieViewer::Errored;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = match self {
            MovieViewer::Selecting => column![
                text("Select movie folder").size(30),
                button("Select folder!")
                    .on_press(Message::FolderSelect)
                    .style(theme::Button::Primary)
            ]
            .width(Length::Shrink),
            MovieViewer::Loading { path } => {
                column![text(format!("Loading movie folder {}", path)).size(30),]
                    .width(Length::Shrink)
            }
            MovieViewer::Loaded { folder } => {
                let mut movie_rows: Vec<Element<Message>> =
                    Vec::with_capacity(folder.movies.capacity());
                let mut current_row: Vec<Element<Message>> = Vec::with_capacity(4);
                let mut i = 4;
                for movie in folder.movies.iter() {
                    current_row.push(
                        container(
                            column![
                                text(&movie.title).size(18),
                                text(&movie.year.to_string()).size(15)
                            ]
                            .padding(6),
                        )
                        .center_x()
                        .style(theme::Container::Bordered)
                        .into(),
                    );
                    i -= 1;
                    if i == 0 {
                        movie_rows.push(row(current_row).spacing(6).into());
                        current_row = Vec::with_capacity(4);
                        i = 4;
                    }
                }
                if !current_row.is_empty() {
                    movie_rows.push(row(current_row).spacing(6).into());
                }
                let movie_grid: Element<_> = column(movie_rows.into()).spacing(6).into();
                column![text(&folder.path).size(15), movie_grid]
            }
            MovieViewer::Errored => {
                column![text("Encountered error").size(30),]
            }
        };
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            //.center_y()
            .padding(8)
            .into()
    }
}
