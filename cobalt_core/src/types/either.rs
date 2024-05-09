

#[derive(Debug, Clone, PartialEq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Either<A, B> {
    pub fn is_left(&self) -> bool {
        match self {
            Either::Left(_) => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Either::Right(_) => true,
            _ => false,
        }
    }

    pub fn take_left(self) -> Option<A> {
        match self {
            Either::Left(a) => Some(a),
            _ => None,
        }
    }

    pub fn take_right(self) -> Option<B> {
        match self {
            Either::Right(b) => Some(b),
            _ => None,
        }
    }

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

    pub fn map_left<C, F: FnOnce(A) -> C>(self, f: F) -> Either<C, B> {
        match self {
            Either::Left(a) => Either::Left(f(a)),
            Either::Right(b) => Either::Right(b),
        }
    }

    pub fn map_right<C, F: FnOnce(B) -> C>(self, f: F) -> Either<A, C> {
        match self {
            Either::Left(a) => Either::Left(a),
            Either::Right(b) => Either::Right(f(b)),
        }
    }

    pub fn as_ref(&self) -> Either<&A, &B> {
        match self {
            Either::Left(a) => Either::Left(a),
            Either::Right(b) => Either::Right(b),
        }
    }

    pub fn as_mut(&mut self) -> Either<&mut A, &mut B> {
        match self {
            Either::Left(a) => Either::Left(a),
            Either::Right(b) => Either::Right(b),
        }
    }

    pub fn map_either<C, D, F: FnOnce(A) -> C, G: FnOnce(B) -> D>(self, f: F, g: G) -> Either<C, D> {
        match self {
            Either::Left(a) => Either::Left(f(a)),
            Either::Right(b) => Either::Right(g(b)),
        }
    }
}