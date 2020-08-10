pub trait TryAsMut<T: ?Sized> {
    fn try_as_mut(&mut self) -> Option<&mut T>;
}

impl<'a, T> TryAsMut<[T]> for &'a mut [T] {
    fn try_as_mut(&mut self) -> Option<&mut [T]> {
        Some(self.as_mut())
    }
}

impl<'a, T> TryAsMut<[T]> for &'a [T] {
    fn try_as_mut(&mut self) -> Option<&mut [T]> {
        None
    }
}
