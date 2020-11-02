use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Template(String);

impl std::ops::Deref for Template {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Path,
            String,
        }
        struct TemplateVisitor;

        impl<'de> serde::de::Visitor<'de> for TemplateVisitor {
            type Value = Template;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .write_str("a map with either `path` pointing to a template file, or `string`")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Template, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                use serde::de;

                let mut path: Option<PathBuf> = None;
                let mut string = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Path => {
                            if path.is_some() {
                                return Err(de::Error::duplicate_field("path"));
                            }
                            path = Some(map.next_value()?);
                        }
                        Field::String => {
                            if string.is_some() {
                                return Err(de::Error::duplicate_field("string"));
                            }
                            string = Some(map.next_value()?);
                        }
                    }
                }

                let string = if let Some(path) = path {
                    std::fs::read_to_string(path).map_err(|e| de::Error::custom(e))?
                } else if let Some(s) = string {
                    s
                } else {
                    return Err(de::Error::custom(
                        "template file not found and no `string` key provided.",
                    ));
                };

                Ok(Template(string))
            }
        }

        const FIELDS: &'static [&'static str] = &["path", "string"];
        deserializer.deserialize_struct("Duration", FIELDS, TemplateVisitor)
    }
}
