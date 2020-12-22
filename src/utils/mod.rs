use enum_as_inner::EnumAsInner;

pub mod graph;
pub mod iter;

// Three spaces of indentation
static INDENT: &str = "   ";

pub mod log {
    use std::fmt::Debug;

    use colored::Colorize;

    use crate::utils::INDENT;

    fn create_indentation(indentation_repeat: usize) -> String {
        std::iter::repeat(INDENT)
            .take(indentation_repeat)
            .collect::<String>()
    }

    pub fn info(t: &str) {
        println!("{}", t.cyan());
    }

    pub fn info_indent(t: &str, indentation_repeat: usize) {
        println!("{}{}", create_indentation(indentation_repeat), t.cyan());
    }

    pub fn error(t: &str) {
        eprintln!("{}", t.red());
    }

    pub fn error_indent(t: &str, indentation_repeat: usize) {
        eprintln!("{}{}", create_indentation(indentation_repeat), t.red());
    }

    pub fn success(t: &str) {
        println!("{}", t.green());
    }

    pub fn success_indent(t: &str, indentation_repeat: usize) {
        println!("{}{}", create_indentation(indentation_repeat), t.green());
    }

    pub fn title(t: &str) {
        println!("{}", t.bold())
    }

    pub fn title_indent(t: &str, indentation_repeat: usize) {
        println!("{}{}", create_indentation(indentation_repeat), t.bold())
    }

    pub fn subtitle(t: &str) {
        println!("{}", t)
    }

    pub fn body<T: Debug>(i: &T) {
        let text = format!("{:#?}", i);
        println!("{}", text.as_str().white());
    }

    pub fn vm_title<T: Debug>(text: &str, i: &T) {
        let title = text.yellow().bold();
        let body = format!("{:?}", i).yellow();
        println!("{}: {}", title, body);
    }

    pub fn vm_subtitle<T: Debug>(text: &str, i: &T) {
        let title = format!("        {}: ", text).as_str().blue().bold();
        println!("{} {}", title, format!("{:?}", i).blue());
    }
}

#[derive(Debug, Clone, Copy, EnumAsInner)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn is_left(&self) -> bool {
        match self {
            Either::Left(_) => true,
            Either::Right(_) => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Either::Left(_) => false,
            Either::Right(_) => true,
        }
    }
}

#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr), *) => {{
        let mut hashmap = std::collections::HashMap::new();
        $(
          hashmap.insert($key, $value);
        )*
        hashmap
    }};
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    // macro creates correct HashMap
    #[test]
    fn hashmap() {
        let standard_map: HashMap<&str, i32> = {
            let mut map = HashMap::new();
            map.insert("one", 1);
            map.insert("two", 2);
            map.insert("three", 3);
            map
        };

        let map_from_macro = hashmap!(
            "one" => 1,
            "two" => 2,
            "three" =>3
        );

        assert_eq!(standard_map, map_from_macro);
    }
}
