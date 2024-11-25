use cruet::string::pluralize;
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use minijinja::Environment;

/// Registers all available filters for a given `Minijinja` environment.
pub fn register_all(env: &mut Environment) {
    env.add_filter("snake_case", snake_case);
    env.add_filter("camel_case", camel_case);
    env.add_filter("kebab_case", kebab_case);
    env.add_filter("pascal_case", pascal_case);
    env.add_filter("lower_camel_case", lower_camel_case);
    env.add_filter("plural", plural);
}

pub fn snake_case(value: String) -> String {
    value.to_snake_case()
}

pub fn camel_case(value: String) -> String {
    value.to_lower_camel_case()
}

pub fn kebab_case(value: String) -> String {
    value.to_kebab_case()
}

pub fn pascal_case(value: String) -> String {
    value.to_pascal_case()
}

pub fn lower_camel_case(value: String) -> String {
    value.to_lower_camel_case()
}

pub fn plural(value: String) -> String {
    pluralize::to_plural(&value)
}