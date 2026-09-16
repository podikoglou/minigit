mod fixtures {
    use minigit::{
        error::ParserContext,
        object::{Object, ObjectType, commit::Identity, tree::TreeEntry},
        storage::object::loose::read_object_compressed,
    };

    #[test]
    fn read_loose_blob() {
        let bytes = include_bytes!("fixtures/objects/blob-1");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose blob object");

        let Object::Blob(blob) = &object else {
            panic!("expected Object::Blob, got {object:?}");
        };

        assert_eq!(blob.0.len(), 151);
        assert_eq!(
            object.hash().unwrap().to_string(),
            "be27a74ddcc0445b1710e25dd8df96fad679a10d"
        );
    }

    #[test]
    fn read_loose_tree() {
        let bytes = include_bytes!("fixtures/objects/tree-1");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose tree object");

        let Object::Tree(tree) = &object else {
            panic!("expected Object::Tree, got {object:?}");
        };

        assert_eq!(tree.entries.len(), 1);
        assert_eq!(
            tree.entries.first_key_value(),
            Some((
                &String::from("README.md"),
                &TreeEntry::new(
                    0o100644,
                    "be27a74ddcc0445b1710e25dd8df96fad679a10d".parse().unwrap()
                )
            ))
        );
    }

    #[test]
    fn read_loose_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-1");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "7bfeab1d89aa800dff6acaae16b3433e7df44fa6"
        );
        assert_eq!(commit.author.0.name, "alex");
        assert_eq!(commit.author.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.committer.0.name, "alex");
        assert_eq!(commit.committer.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.description, "add readme\n");
    }

    #[test]
    fn read_loose_non_root_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-2");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "f1596e78773e04b539660b10d75c02928ff38703"
        );
        assert_eq!(
            commit.parents,
            vec!["497b458b242ad074b046386ec56b9f19361b2691".parse().unwrap()]
        );
        assert_eq!(commit.author.0.name, "alex");
        assert_eq!(commit.author.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.committer.0.name, "alex");
        assert_eq!(commit.committer.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.description, "7133\n");
    }

    #[test]
    fn read_loose_merge_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-3");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "45bf6921d3d209c3ca2623277bc41868908c6831"
        );
        assert_eq!(
            commit.parents,
            vec![
                "a3dbc49f01da99433914156f54c847c162256cc0".parse().unwrap(),
                "3e66aa515edae750fc16edabea2291a458069db0".parse().unwrap(),
            ]
        );
        assert_eq!(commit.author.0.name, "alex");
        assert_eq!(commit.author.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.committer.0.name, "alex");
        assert_eq!(commit.committer.0.email, "alex.podikoglou@gmail.com");
        assert_eq!(commit.description, "Merge branch 'feature'\n");
    }

    #[test]
    fn read_loose_signed_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-4");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "d6de8cca32ce6db2c44e2f16fac72ed5d7f6ec2a"
        );
        assert_eq!(
            commit.parents,
            vec![
                "759c3dfd40f9d4833b86ca587f1789b4c10772e9".parse().unwrap(),
                "33ba4daaf56cdd499f4cb8960eb0020db7619616".parse().unwrap(),
            ]
        );
        assert_eq!(commit.author.0.name, "Laurens Kuiper");
        assert_eq!(commit.author.0.email, "laurens@ducklabs.com");
        assert_eq!(commit.committer.0.name, "GitHub");
        assert_eq!(commit.committer.0.email, "noreply@github.com");
        assert!(commit.gpg_signature.is_some());
        assert_eq!(
            commit.description,
            "`COPY` with `ORDER_BY` without `PARTITION_BY` (#24525)\n\nI didn't get around to doing this, but it was surprisingly a very small\nchange.\n\nCC @Tmonster\n"
        );
    }

    #[test]
    fn read_loose_commit_with_extra_properties() {
        let bytes = include_bytes!("fixtures/objects/commit-5");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "512310236d7623ac3a86fc96c24b94280b44bb41"
        );
        assert_eq!(
            commit.parents,
            vec!["18b593788d6ebd548bcf55b18cc8f3e15d5fb4c3".parse().unwrap()]
        );
        assert_eq!(commit.author.0.name, "Nicholas Junge");
        assert_eq!(commit.author.0.email, "nicho.junge@gmail.com");
        assert_eq!(commit.committer.0.name, "Nicholas Junge");
        assert_eq!(commit.committer.0.email, "nicho.junge@gmail.com");
        assert_eq!(
            commit.extra,
            vec![(
                "change-id".to_string(),
                "xnxouqnvmpzvuvkotwynowookslovtno".to_string()
            )]
        );
        assert_eq!(
            commit.description,
            "CMake: Set the extension base directory to the duckdb module dir\n\nDuckDB's extension-patch step hardcodes `${CMAKE_SOURCE_DIR}`, which only equals the\nsubmodule root in a standalone build. When DuckDB is pulled in via `add_subdirectory`,\nit resolves to the parent project's root, which causes the patch script and patch-dir\nlookups to happen in the wrong place.\n\nI ran into this because I'm vendoring duckDB as a submodule for a Python bindings project,\nand statically link extensions from the submodule checkout in the build process.\n"
        );
    }

    #[test]
    fn read_loose_commit_with_empty_email() {
        let bytes = include_bytes!("fixtures/objects/commit-6");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose commit object");

        let Object::Commit(commit) = &object else {
            panic!("expected Object::Commit, got {object:?}");
        };

        assert_eq!(
            commit.tree.to_string(),
            "bf2c4695aac832581edbce6bf3e303db98181a71"
        );
        assert_eq!(
            commit.parents,
            vec!["09e2d957342607904124ebab892be70b0ecf9a10".parse().unwrap()]
        );
        assert_eq!(commit.author.0.name, "Virgiel");
        assert_eq!(commit.author.0.email, "");
        assert_eq!(commit.committer.0.name, "Virgiel");
        assert_eq!(commit.committer.0.email, "Virgiel@users.noreply.github.com");
        assert_eq!(
            commit.description,
            "duckdb_interrupt & duckdb_query_progress\n"
        );
    }

    #[test]
    fn read_loose_tag() {
        let bytes = include_bytes!("fixtures/objects/tag-1");
        let object = read_object_compressed(&bytes[..], ParserContext::None)
            .expect("should be able to read loose tag object");

        let Object::Tag(tag) = &object else {
            panic!("expected Object::Tag, got {object:?}");
        };

        assert_eq!(
            tag.target.0.to_string(),
            "ab2b0330d1eb193e5e339c94197998e9baaa8846"
        );
        assert_eq!(tag.target.1, ObjectType::Commit);
        assert_eq!(tag.name, "v0.0.1");
        assert_eq!(tag.description, "0.0.1! :D\n");
        assert_eq!(
            tag.tagger.0,
            Identity::new("alex".to_string(), "alex.podikoglou@gmail.com".to_string())
        );
    }
}

mod roundtrip {
    use hegel::Generator;
    use hegel::TestCase;
    
    use minigit::error::ParserContext;
    use minigit::object::Object;
    
    
    use minigit::storage::object::loose::WriteLoose;
    use minigit::storage::object::loose::read_object;

    mod generators {
        use crate::roundtrip::generators;
        
        use hegel::Generator;
        use hegel::TestCase;
        use hegel::extras::chrono::datetimes;
        use hegel::generators as gs;
        use minigit::object::ObjectType;
        use minigit::object::blob::Blob;
        use minigit::object::commit::Commit;
        use minigit::object::commit::CommitProperty;
        use minigit::object::commit::Identity;
        use minigit::object::hash::ObjectHash;
        use minigit::object::tag::Tag;
        use minigit::object::tree::Tree;
        use minigit::object::tree::TreeEntry;
        use strum::VariantArray;

        #[hegel::composite]
        pub fn hash(tc: &TestCase) -> ObjectHash {
            tc.draw(
                gs::arrays(gs::integers())
                    .map(|bytes: [u8; 20]| ObjectHash::from(bytes))
                    .print_as_debug(),
            )
        }

        #[hegel::composite]
        pub fn blob(tc: &TestCase) -> Blob {
            tc.draw(gs::binary().map(Blob).print_as_debug())
        }

        #[hegel::composite]
        pub fn tree_entry(tc: &TestCase) -> TreeEntry {
            let mode: u16 = tc.draw(gs::integers());
            let hash = tc.draw(generators::hash().print_as_debug());

            TreeEntry { mode, object: hash }
        }

        #[hegel::composite]
        pub fn tree(tc: &TestCase) -> Tree {
            let key = gs::text();
            let value = generators::tree_entry().print_as_debug();

            tc.draw(gs::btree_maps(key, value).map(Tree::new).print_as_debug())
        }

        #[hegel::composite]
        pub fn identity(tc: &TestCase) -> Identity {
            let name = tc.draw(gs::text());

            // TODO: consider gs::text, since we don't do email validation, thus we accenpt anything
            let email = tc.draw(gs::emails());

            Identity::new(name, email)
        }

        #[hegel::composite]
        pub fn property(tc: &TestCase) -> CommitProperty {
            tc.draw(gs::tuples!(gs::text(), gs::text()))
        }

        #[hegel::composite]
        pub fn object_type(tc: &TestCase) -> ObjectType {
            tc.draw(gs::sampled_from(ObjectType::VARIANTS).print_as_debug())
        }

        #[hegel::composite]
        pub fn commit(tc: &TestCase) -> Commit {
            let tree = tc.draw(hash().print_as_debug());
            let parents = tc.draw(gs::vecs(hash()).print_as_debug());
            let author = tc.draw(gs::tuples!(identity(), datetimes()).print_as_debug());
            let committer = tc.draw(gs::tuples!(identity(), datetimes()).print_as_debug());
            let gpg_signature = tc.draw(gs::optional(gs::text()).print_as_debug());
            let extra = tc.draw(gs::vecs(property()).print_as_debug());
            let description = tc.draw(gs::text());

            Commit::new(
                tree,
                parents,
                author,
                committer,
                gpg_signature,
                extra,
                description,
            )
        }

        #[hegel::composite]
        pub fn tag(tc: &TestCase) -> Tag {
            let target = tc.draw(gs::tuples!(hash(), object_type()).print_as_debug());
            let name = tc.draw(gs::text());
            let tagger = tc.draw(gs::tuples!(identity(), datetimes()).print_as_debug());
            let description = tc.draw(gs::text());

            Tag::new(target, name, tagger, description)
        }
    }

    #[hegel::test]
    fn roundtrip_blob(tc: TestCase) {
        let object = tc.draw(generators::blob().map(Object::from).print_as_debug());

        // write to buffer
        let mut buf: Vec<u8> = Vec::new();
        object.write_loose(&mut buf).unwrap();

        // read back
        let read_blob = read_object(&buf[..], ParserContext::None).unwrap();

        assert_eq!(read_blob, object);
    }

    #[hegel::test]
    #[ignore]
    fn roundtrip_tree(tc: TestCase) {
        let object = tc.draw(generators::tree().map(Object::from).print_as_debug());

        // write to buffer
        let mut buf: Vec<u8> = Vec::new();
        object.write_loose(&mut buf).unwrap();

        // read back
        let read_tree = read_object(&buf[..], ParserContext::None).unwrap();

        assert_eq!(read_tree, object);
    }

    #[hegel::test]
    #[ignore]
    fn roundtrip_commit(tc: TestCase) {
        let object = tc.draw(generators::commit().map(Object::from).print_as_debug());

        // write to buffer
        let mut buf: Vec<u8> = Vec::new();
        object.write_loose(&mut buf).unwrap();

        // read back
        let read_tree = read_object(&buf[..], ParserContext::None).unwrap();

        assert_eq!(read_tree, object);
    }

    #[hegel::test]
    #[ignore]
    fn roundtrip_tag(tc: TestCase) {
        let object = tc.draw(generators::tag().map(Object::from).print_as_debug());

        // write to buffer
        let mut buf: Vec<u8> = Vec::new();
        object.write_loose(&mut buf).unwrap();

        // read back
        let read_tag = read_object(&buf[..], ParserContext::None).unwrap();

        assert_eq!(read_tag, object);
    }
}
