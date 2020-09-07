#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    DisplayCreation(#[from] glium::backend::glutin::DisplayCreationError),
    #[error("{0}")]
    SwapBuffers(#[from] glium::SwapBuffersError),
}

pub type Result<T> = std::result::Result<T, Error>;
