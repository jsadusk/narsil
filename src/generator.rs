pub trait Generator {
    type Item;
    type Iterator: Iterator<Item=Self::Item>;

    fn iter(&self) -> Self::Iterator;
}

