// Define a struct that will hold the given iterator and will provide pair iteration.
struct PairIteratorImpl<I: Iterator>(I);

// Impl Iterator for it..
impl<I> Iterator for PairIteratorImpl<I>
where
    I: Iterator,
{
    type Item = (Option<I::Item>, Option<I::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.0.next();
        let second = self.0.next();
        // If atleast one element is present, return the pair, else return None.
        if first.is_some() {
            Some((first, second))
        } else {
            None
        }
    }
}

// Define a trait that adds functionality to returns PairIterator on any iterator. This adds `pair()` method on any iterator.
trait PairIterator: Iterator {
    fn pairs(self) -> PairIteratorImpl<Self>
    where
        Self: Sized;
}

// Impl trait to return a PairIterator impl on any iterator
impl<I> PairIterator for I
where
    I: Iterator,
{
    fn pairs(self) -> PairIteratorImpl<Self>
    where
        Self: Sized,
    {
        PairIteratorImpl(self)
    }
}

pub fn run() {
    let d = [1, 2, 3];
    for pair in d.iter().pairs() {
        println!("{:?}", pair);
    }
    let d = ["a", "b", "c", "d"];
    for pair in d.iter().pairs() {
        println!("{:?}", pair);
    }
}
