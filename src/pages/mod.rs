mod admin;
pub(crate) mod exercises;
mod history;
mod home;
mod log_workout;
mod login;
mod profile;
mod wod;

pub use admin::{AdminExercisesPage, AdminPage};
pub use exercises::ExercisesPage;
pub use history::HistoryPage;
pub use home::HomePage;
pub use log_workout::LogWorkoutPage;
pub use login::LoginPage;
pub use profile::ProfilePage;
pub use wod::WodPage;
