mod fixtures {
    use minigit::{error::ParserContext, object::Object, storage::object::loose::read_object};

    #[test]
    fn test_read_loose_blob() {
        let bytes = include_bytes!("fixtures/objects/blob-1");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_tree() {
        let bytes = include_bytes!("fixtures/objects/tree-1");
        let object = read_object(&bytes[..], ParserContext::None)
            .expect("should be able to read loose tree object");

        let Object::Tree(tree) = &object else {
            panic!("expected Object::Tree, got {object:?}");
        };

        assert_eq!(tree.entries.len(), 1);
        // assert_eq!(
        //     tree.entries.first_key_value(),
        //     Some((
        //         &String::from("README.md"),
        //         TreeEntry::new(0o100644 /* TODO */,)
        //     ))
        // );
    }

    #[test]
    fn test_read_loose_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-1");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_non_root_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-2");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_merge_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-3");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_signed_commit() {
        let bytes = include_bytes!("fixtures/objects/commit-4");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_commit_with_extra_properties() {
        let bytes = include_bytes!("fixtures/objects/commit-5");
        let object = read_object(&bytes[..], ParserContext::None)
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
    fn test_read_loose_commit_with_empty_email() {
        let bytes = include_bytes!("fixtures/objects/commit-6");
        let object = read_object(&bytes[..], ParserContext::None)
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
}

mod roundtrip {
    use hegel::TestCase;
    use hegel::generators as gs;
    use minigit::error::ParserContext;
    use minigit::object::Object;
    use minigit::object::blob::Blob;
    use minigit::storage::object::loose::WriteLoose;
    use minigit::storage::object::loose::read_object;

    #[hegel::test]
    fn roundtrip_blob(tc: TestCase) {
        // construct blob
        let data = tc.draw(gs::binary());

        let blob = Blob(data);
        let object: Object = blob.into();

        // write to buffer
        let mut buf: Vec<u8> = Vec::new();
        object.write_loose(&mut buf).unwrap();

        // read back
        let read_blob = read_object(&buf[..], ParserContext::None).unwrap();

        assert_eq!(object, read_blob);
    }
}
