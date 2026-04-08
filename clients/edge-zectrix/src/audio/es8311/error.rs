#[derive(Debug)]
pub enum Error<E> {
    BusError(E),
    InvalidConfiguration,
}
