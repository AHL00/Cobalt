use serde::{de::VariantAccess, Serialize};



#[derive(Debug, Clone, PartialEq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: Serialize, B: Serialize> Serialize for Either<A, B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Either::Left(a) => serializer.serialize_newtype_variant("Either", 0, "Left", a),
            Either::Right(b) => serializer.serialize_newtype_variant("Either", 1, "Right", b),
        }
    }
}

impl<'de, A: serde::Deserialize<'de>, B: serde::Deserialize<'de>> serde::Deserialize<'de> for Either<A, B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct EitherVisitor<A, B> {
            _marker: std::marker::PhantomData<(A, B)>,
        }

        impl<'de, A: serde::Deserialize<'de>, B: serde::Deserialize<'de>> serde::de::Visitor<'de> for EitherVisitor<A, B> {
            type Value = Either<A, B>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Either<A, B>")
            }

            fn visit_enum<C>(self, data: C) -> Result<Self::Value, C::Error>
            where
                C: serde::de::EnumAccess<'de>,
            {
                let (variant, value) = data.variant::<&str>()?;
                match variant {
                    "Left" => {
                        let value = value.newtype_variant::<A>()?;
                        Ok(Either::Left(value))
                    }
                    "Right" => {
                        let value = value.newtype_variant::<B>()?;
                        Ok(Either::Right(value))
                    }
                    _ => Err(serde::de::Error::unknown_variant(variant, &["Left", "Right"])),
                }
            }
        }

        deserializer.deserialize_enum("Either", &["Left", "Right"], EitherVisitor { _marker: std::marker::PhantomData })
    }
}

#[test]
fn test_either_serialize() {
    use serde_yaml;

    let either: Either<u32, String> = Either::Left(42);
    let serialized = serde_yaml::to_string(&either).unwrap();
    assert_eq!(serialized, "Left: 42\n");
    
    let either: Either<u32, String> = Either::Right("Hello, World!".to_string());
    let serialized = serde_yaml::to_string(&either).unwrap();
    assert_eq!(serialized, "Right: \"
    Hello, World!\"\n");
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