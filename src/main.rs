use iced::futures;
use iced::widget::{self, column, container, image, row, text, button};
use iced::{Alignment, Application, Color, Command, Element, Length, Settings, Theme};
use native_dialog::{FileDialog};

fn main() -> iced::Result {
    MovieViewer::run(Settings::default())
}

#[derive(Debug, Clone)]
struct MovieFolder {
    path: String,
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
    Title: String,
    Year: u32,
    Rated: MovieRating,
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
    async fn load(path: String) -> Result<MovieFolder, Error>{
        Ok(MovieFolder{path})
    }
}

#[derive(Debug, Clone)]
enum MovieMessage {
    FolderSelect,
    FolderSelected(Result<String, Error>),
    FolderLoaded(Result<MovieFolder, Error>),
}

impl Application for MovieViewer {
    
    type Message = MovieMessage;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (MovieViewer, Command<MovieMessage>) {
        (MovieViewer::Selecting,
        Command::none())
    }

    fn title(&self) -> String {
        "MovieViewer".to_string()
    }

    fn update(&mut self, message: MovieMessage) -> Command<MovieMessage> {
        match message {
            MovieMessage::FolderSelect => {
                Command::perform(MovieFolder::select(), MovieMessage::FolderSelected)
            },
            MovieMessage::FolderSelected(Err(_error)) => {
                *self = MovieViewer::Errored;
                Command::none()
            },
            MovieMessage::FolderSelected(Ok(path)) => {
                *self = MovieViewer::Loading { path: path.clone() };
                Command::perform(MovieFolder::load(path), MovieMessage::FolderLoaded )
            },
            MovieMessage::FolderLoaded(Ok(folder)) => {
                *self = MovieViewer::Loaded{folder};
                Command::none()
            },
            MovieMessage::FolderLoaded(Err(_error)) => {
                *self = MovieViewer::Errored;
                Command::none()
            } 
        }
    }

    fn view(&self) -> Element<MovieMessage> {
        let content = match self {
            MovieViewer::Selecting => {
                column![
                    text("Select movie folder").size(30),
                    button("Select folder!").on_press(MovieMessage::FolderSelect)
                ].width(Length::Shrink)
            },
            MovieViewer::Loading { path } => {
                column![
                    text(format!("Loading movie folder {}", path)).size(30),
                ].width(Length::Shrink)
            },
            MovieViewer::Loaded { folder } => {
                column![
                    text(format!("Viewing movie folder {}", folder.path)).size(20),
                ]
            },
            MovieViewer::Errored => {
                column![
                    text("Encountered error").size(30),
                ]
            }
        };
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

}
