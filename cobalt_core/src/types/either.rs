

#[derive(Debug, Clone, PartialEq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Either<A, B> {
    pub fn left(&self) -> Option<&A> {
        match self {
            Either::Left(a) => Some(a),
            _ => None,
        }
    }

    pub fn right(&self) -> Option<&B> {
        match self {
            Either::Right(b) => Some(b),
            _ => None,
        }
    }

    pub fn left_mut(&mut self) -> Option<&mut A> {
        match self {
            Either::Left(a) => Some(a),
            _ => None,
        }
    }

    pub fn right_mut(&mut self) -> Option<&mut B> {
        match self {
            Either::Right(b) => Some(b),
            _ => None,
        }
    }
}