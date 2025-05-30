use std::convert::Infallible;

use crate::ser;

/// A serializer that does nothing. This is useful as a sub-type for serializers which only do certain parts of [`ser::Serializer`].
pub struct NoopDeSerializer<Ok, Err> {
    _infallible: Infallible,
    _marker: std::marker::PhantomData<(Ok, Err)>,
}

impl ser::Error for Infallible {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> crate::ser::SerializeSeq for NoopDeSerializer<Ok, Err> {
    type Ok = Ok;

    type Error = Err;

    fn serialize_element<V: ser::Serialize>(&mut self, _: &V) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> crate::ser::SerializeAttributes for NoopDeSerializer<Ok, Err> {
    type Ok = Ok;

    type Error = Err;

    fn serialize_attribute<A: ser::SerializeAttribute>(
        &mut self,
        _a: &A,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> crate::ser::SerializeElementAttributes for NoopDeSerializer<Ok, Err> {
    type ChildrenSerializeSeq = NoopDeSerializer<Ok, Err>;

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok, Err: ser::Error> crate::ser::SerializeElement for NoopDeSerializer<Ok, Err> {
    type Ok = Ok;

    type Error = Err;

    type ChildrenSerializeSeq = NoopDeSerializer<Ok, Err>;

    type SerializeElementAttributes = NoopDeSerializer<Ok, Err>;

    fn include_prefix(
        &mut self,
        _should_enforce: crate::ser::IncludePrefix,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn preferred_prefix(
        &mut self,
        _preferred_prefix: Option<crate::Prefix<'_>>,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_attributes(self) -> Result<Self::SerializeElementAttributes, Self::Error> {
        todo!()
    }

    fn serialize_children(self) -> Result<Self::ChildrenSerializeSeq, Self::Error> {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
