Feature: Import/install files from prepared image folder

    Most of the time when we build package for Exherbo/Gentoo we specify
    designated folder right at the installation phase were building
    infrastructure should place all files as if they were installed into root.

    Background:
        Given sample with minimum content

    Scenario: Install empty image folder
        Given directory /tmp/image
        When run ndbam-import --image ${root}/tmp/image empty-package
        Then success
        When run ndbam-check empty-package
        Then success

    Scenario: Install file in a root
        Given file /tmp/image/hello.md
            """
            Hello Exherbo!
            """
        When run ndbam-import --image ${root}/tmp/image just-file
        Then success
        When run ndbam-check -v just-file
        Then success
        And output contains: cb4e2f2f8fddf2d59373bf01856e503e
        And output contains: Size: 14 B
        And file /hello.md exists
            """
            Hello Exherbo!
            """
        But no file /tmp/image/hello.md exist

    Scenario: Install file in a sub-dir
        Given file /tmp/image/docs/hello.md
            """
            Hello Exherbo!
            """
        When run ndbam-import --image ${root}/tmp/image just-file
        Then success
        When run ndbam-check -v just-file
        Then success
        And output contains: cb4e2f2f8fddf2d59373bf01856e503e
        And file /docs/hello.md exists
        But no file /tmp/image/docs/hello.md exist

    Scenario: Install empty directory
        Given dir /tmp/image/empty-dir
        When run ndbam-import --image ${root}/tmp/image just-dir
        Then success
        And directory /empty-dir exists
        But no directory /tmp/image/hello.md exist

    Scenario: Install symlink
        Given dir /2019
        And symlink /tmp/image/latest to 2019
        When run ndbam-import --image ${root}/tmp/image years
        Then success
        And symlink /latest to 2019 exists
        But no symlink /tmp/image/latest exist
