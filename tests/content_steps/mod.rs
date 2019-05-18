use super::*;
use std::fs;

steps!(Env => {
    given regex r"^file (.+)$" (PathBuf) |world, path, step| {
        let real_path = world.real_path(&path);
        create_dir_for(&real_path);
        if let Some(content) = step.docstring() {
            assert!(!content.contains('<') && !content.contains('>'),
            "variables are not yet supported. cucumber test skipped"); // magic trail to skip
            fs::write(&real_path, content)
        } else {
            fs::write(&real_path, "dummy")
        }.expect(format!("write to {:?} (original {:?})", &real_path, &path).as_str());
    };

    given regex r"^directory (.+)$" (PathBuf) |world, path, _step| {
        fs::create_dir_all(world.real_path(&path)).unwrap();
    };

    given regex r"^symlink (.+) to (.+)$" (PathBuf, PathBuf) |world, path, target, _step| {
        let real_path = world.real_path(&path);
        create_dir_for(&real_path);
        fs::soft_link(&target, &real_path)
            .expect(format!("symlink at {:?} {:?}", &real_path, &path).as_str());
    };
});

fn create_dir_for(path: &Path) {
    path.parent()
        .map(|parent| fs::create_dir_all(parent).unwrap());
}
