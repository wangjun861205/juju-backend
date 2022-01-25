mod test {
    use crate::serde::Serialize;
    #[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
    struct User {
        id: i32,
        role: String,
    }

    #[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
    struct Resource {
        id: i32,
        owner: User,
    }
    #[tokio::test]
    async fn test_casbin() {
        use crate::casbin::prelude::*;
        let mut e = Enforcer::new("src/privilege/casbin.conf", "src/privilege/policies.csv")
            .await
            .unwrap();
        e.enable_log(true);
        let res = e
            .enforce((
                User {
                    id: 1,
                    role: "admin".to_owned(),
                },
                Resource {
                    id: 1,
                    owner: User {
                        id: 3,
                        role: "user".to_owned(),
                    },
                },
                "read",
            ))
            .unwrap();
        println!("{}", res);
    }
}
