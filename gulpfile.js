const fs = require("fs");
const path = require("path");
const gulp = require("gulp");
const babel = require("gulp-babel");
const ts = require("gulp-typescript");
const cmd = require("child_process");
const tsProject = ts.createProject("tsconfig.json");
const logger = require("clear-logger").default;

gulp.task("build_pre", (done) => {
    const outdir = path.join(process.cwd(), "pkg");
    if (fs.existsSync(outdir)) {
        fs.rmSync(outdir, { recursive: true });
    }
    done();
})

gulp.task("build_main",() => {
    const packageName = process.cwd().split(path.sep).pop();
    cmd.exec(`wasm-pack build --target nodejs --out-name index`, (e, stdout, stderr) => {
        const _logger = logger.customName(packageName);
        if (stderr.search('Finished') !== -1) {
        //   _logger.success(stderr);
        } else {
          _logger.debug(e, false);
          _logger.debug(stdout, false);
          _logger.debug(`${stderr}`, false);
          process.exit(1);
        }
    });

    const tsResult = tsProject.src()
    .pipe(babel())
    .pipe(gulp.dest("./pkg/res_src"));

    return tsResult;
})

gulp.task("build_post", (done) => {
    done();
})

gulp.task("build", gulp.series("build_pre","build_main", "build_post"));

