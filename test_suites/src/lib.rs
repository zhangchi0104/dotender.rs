#[allow(unused_macros)]
macro_rules! path {
    ($($parts:ident)/+) => {
        vec!($(stringify!($parts)),+).iter().collect::<std::path::PathBuf>()
    };
    (../$($parts:ident)/+) => {
        vec!("..", $(stringify!($parts)),+).iter().collect::<std::path::PathBuf>()
    };
    (./$($parts:ident)/+) => {
        path!($($parts)/+)
    };
     (/$($parts:ident)/+) => {
        vec!("/", $(stringify!($parts)),+).iter().collect::<std::path::PathBuf>()
    };
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    #[test]
    fn test_path_macro() {
        let p = path!(data / name);
        assert_eq!(p, PathBuf::from("data/name"));
        let p = path!(./ data / name);
        assert_eq!(p, PathBuf::from("data/name"));
        let p = path!(/data/name);
        assert_eq!(p, PathBuf::from("/data/name"));
    }
}
