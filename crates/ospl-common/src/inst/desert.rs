use serde::{Serialize, Deserialize};
use serde::de::Deserializer;

use crate::inst::{RT, RV, RuntimeValue};

impl Serialize for RuntimeValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut tup = serializer.serialize_tuple(2)?;

        tup.serialize_element(&self.tag)?;

        unsafe {
            match self.tag {
                RT::Int => tup.serialize_element(&self.data.int)?,
                RT::Addr => tup.serialize_element(&self.data.address)?,
                RT::Float => tup.serialize_element(&self.data.float)?,
                RT::Bool => tup.serialize_element(&self.data.bool)?,
                RT::Char => tup.serialize_element(&self.data.char)?,

                RT::Str => tup.serialize_element(&*self.data.str)?,
                RT::List => tup.serialize_element(&*self.data.list)?,
                RT::Func => tup.serialize_element(&*self.data.func)?,
                RT::Scope => tup.serialize_element(&*self.data.scope)?,

                RT::ForeignLib | RT::ForeignFn => {
                    tup.serialize_element(&self.data.foreign)?
                }

                RT::Nul | RT::Undefined => {
                    tup.serialize_element(&())?
                }
            }
        }

        tup.end()
    }
}

impl<'de> Deserialize<'de> for RuntimeValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = RuntimeValue;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "RuntimeValue tuple (tag, data)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let tag: RT = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::custom("missing tag"))?;

                let value = match tag {
                    RT::Int => RuntimeValue {
                        tag,
                        data: RV {
                            int: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Addr => RuntimeValue {
                        tag,
                        data: RV {
                            address: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Float => RuntimeValue {
                        tag,
                        data: RV {
                            float: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Bool => RuntimeValue {
                        tag,
                        data: RV {
                            bool: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Char => RuntimeValue {
                        tag,
                        data: RV {
                            char: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Str => RuntimeValue {
                        tag,
                        data: RV {
                            str: std::mem::ManuallyDrop::new(seq.next_element()?.unwrap()),
                        },
                    },

                    RT::List => RuntimeValue {
                        tag,
                        data: RV {
                            list: std::mem::ManuallyDrop::new(seq.next_element()?.unwrap()),
                        },
                    },

                    RT::Func => RuntimeValue {
                        tag,
                        data: RV {
                            func: std::mem::ManuallyDrop::new(seq.next_element()?.unwrap()),
                        },
                    },

                    RT::Scope => RuntimeValue {
                        tag,
                        data: RV {
                            scope: std::mem::ManuallyDrop::new(seq.next_element()?.unwrap()),
                        },
                    },

                    RT::ForeignLib | RT::ForeignFn => RuntimeValue {
                        tag,
                        data: RV {
                            foreign: seq.next_element()?.unwrap(),
                        },
                    },

                    RT::Nul | RT::Undefined => RuntimeValue {
                        tag,
                        data: RV { nothing: () },
                    },
                };

                Ok(value)
            }
        }

        deserializer.deserialize_tuple(2, V)
    }
}