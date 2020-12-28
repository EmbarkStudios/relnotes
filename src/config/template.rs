use std::path::PathBuf;

use serde::Deserialize;

const DEFAULT_TEMPLATE: &str = "\
# {{ title }} {{ version }} ({{ date }})

{% for pr in prs -%}
- [{{ pr.title }}]({{ pr.html_url }})
{% endfor %}

{%- for title, prs in categories %}
## {{ title }}

    {% for pr in prs %}
- [{{ pr.title }}]({{ pr.html_url }})
    {%- endfor %}
{% endfor %}

{%- for include in includes %}
## {{ include.title }}

    {% for title, prs in include.categories %}
### {{ title }}

        {%- for pr in prs %}
- [{{ pr.title }}]({{ pr.html_url }})
        {%- endfor %}

    {%- endfor -%}

    {%- for pr in include.prs %}
- [{{ pr.title }}]({{ pr.html_url }})
    {%- endfor %}

{%- endfor %}

## Contributors

{% for contributor in contributors | sort(attribute=\"login\", case_sensitive=\"false\") %}
- [{{ contributor.login }}]({{ contributor.html_url }})
{%- endfor %}

";

#[derive(Clone, Debug)]
pub struct Template(String);

impl Default for Template {
    fn default() -> Self {
        Self(String::from(DEFAULT_TEMPLATE))
    }
}

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
                    DEFAULT_TEMPLATE.into()
                };

                Ok(Template(string))
            }
        }

        const FIELDS: &'static [&'static str] = &["path", "string"];
        deserializer.deserialize_struct("Duration", FIELDS, TemplateVisitor)
    }
}
