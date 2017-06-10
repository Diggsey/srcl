pub trait SliceExt {
    type Elem;
    fn fill_copy(&mut self, value: Self::Elem) where Self::Elem: Copy;
}

impl<T> SliceExt for [T] {
    type Elem = T;

    fn fill_copy(&mut self, value: Self::Elem) where Self::Elem: Copy {
        for elem in self {
            *elem = value;
        }
    }
}
