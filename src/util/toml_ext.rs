use toml::Value;

// --
// -- TomlExt
// --

pub trait TomlExt: Sized {
    fn into_value(self) -> Value;

    fn with_overrides(self, overrides: impl Into<Value>) -> Value {
        let base = self.into_value();
        let overrides = overrides.into();

        match (base, overrides) {
            (Value::Table(mut tab_l), Value::Table(tab_r)) => {
                for (key, val) in tab_r.into_iter() {
                    match tab_l.entry(key) {
                        toml::map::Entry::Vacant(entry) => {
                            entry.insert(val);
                        }
                        toml::map::Entry::Occupied(mut entry) => {
                            let old_val = entry.insert(Value::Boolean(false));
                            entry.insert(old_val.with_overrides(val));
                        }
                    }
                }

                tab_l.into()
            }
            (_, r) => r,
        }
    }
}

impl<T> TomlExt for T
where
    T: Into<Value>,
{
    fn into_value(self) -> Value {
        self.into()
    }
}

// --
// -- TomlBuilder
// --

pub struct TomlBuilder {
    inner: Value,
}

impl Default for TomlBuilder {
    fn default() -> Self {
        Self {
            inner: Value::Table(toml::Table::default()),
        }
    }
}

impl TomlBuilder {
    pub fn build(self) -> Value {
        self.inner
    }

    pub fn with_opt<const N: usize, T: Into<Value>>(
        self,
        key: [&'static str; N],
        opt: Option<T>,
    ) -> Self {
        let Some(val) = opt else { return self };

        let as_override: Value = key.into_iter().rev().fold(val.into(), |acc, key| {
            [(key.to_string(), acc)]
                .into_iter()
                .collect::<toml::Table>()
                .into()
        });

        Self {
            inner: self.inner.with_overrides(as_override),
        }
    }
}
