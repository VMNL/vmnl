# List available recipes.
default:
    @just --list

# Build an example without opening a window.
build example='d2_shapes':
    cargo build -p '{{ example }}'

# Build every workspace target.
build-workspace:
    cargo build --workspace --all-targets

# Run an example.
run example='d2_shapes':
    cargo run -p '{{ example }}'

# Run the headless test suites.
[no-exit-message]
test:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-total test

# Run unit tests.
[no-exit-message]
test-unit:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report unit

# Run public API tests.
[no-exit-message]
test-api:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report api

# Run smoke tests.
[no-exit-message]
test-smoke:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report smoke

# Run portable GLFW error conversion and Null-backend tests.
[no-exit-message]
test-platform:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report platform

# Compile all platform probes without running a native backend.
[no-exit-message]
test-platform-compile:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report platform-compile

# Run only the isolated GLFW Null-backend probe.
[no-exit-message]
test-platform-null:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report platform-null

# Run the Wayland contract against the caller-provided compositor.
test-platform-wayland:
    VMNL_PLATFORM_TEST_BACKEND=wayland cargo test -p vmnl-platform-tests --test backend_contract -- --ignored --nocapture

# Run the X11 contract against the caller-provided X server and EWMH window manager.
test-platform-x11:
    VMNL_PLATFORM_TEST_BACKEND=x11 cargo test -p vmnl-platform-tests --test backend_contract -- --ignored --nocapture

# Compile GPU tests without running them.
[no-exit-message]
test-gpu-compile:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report gpu-compile

# Run GPU/display tests.
[no-exit-message]
test-gpu:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report gpu

# Run Rustdoc examples.
[no-exit-message]
doctest:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-report doctest

# Run one test suite and print a summary while preserving Cargo output and exit status.
[no-exit-message]
_test-report suite:
    #!/usr/bin/env bash
    set -uo pipefail

    if [[ -n ${NO_COLOR+x} ]]; then
        color_args=(--color never)
        cyan=''
        green=''
        red=''
        yellow=''
        reset=''
    elif [[ -t 1 && ${TERM:-dumb} != dumb ]]; then
        color_args=(--color always)
        cyan=$'\033[1;36m'
        green=$'\033[1;32m'
        red=$'\033[1;31m'
        yellow=$'\033[1;33m'
        reset=$'\033[0m'
    else
        color_args=()
        cyan=''
        green=''
        red=''
        yellow=''
        reset=''
    fi

    case '{{ suite }}' in
        api)
            suite_name='API'
            suite_scope='public headless facade'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test -p vmnl-api-tests)
            ;;
        doctest)
            suite_name='RUSTDOC'
            suite_scope='documentation examples'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test --workspace --all-features --doc)
            ;;
        gpu)
            suite_name='GPU'
            suite_scope='Vulkan + GLFW display required'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test -p vmnl-gpu-tests -- --ignored)
            ;;
        gpu-compile)
            suite_name='GPU'
            suite_scope='compile only; no Vulkan frame is executed'
            suite_kind='compile'
            test_command=(cargo "${color_args[@]}" test -p vmnl-gpu-tests --no-run)
            ;;
        platform)
            suite_name='PLATFORM'
            suite_scope='error conversion + GLFW Null backend'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test -p vmnl-platform-tests)
            ;;
        platform-compile)
            suite_name='PLATFORM'
            suite_scope='compile only; no native backend is executed'
            suite_kind='compile'
            test_command=(cargo "${color_args[@]}" test -p vmnl-platform-tests --no-run)
            ;;
        platform-null)
            suite_name='PLATFORM'
            suite_scope='GLFW Null backend'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test -p vmnl-platform-tests --test native_window)
            ;;
        smoke)
            suite_name='SMOKE'
            suite_scope='windowless executable startup'
            suite_kind='smoke'
            test_command=(cargo "${color_args[@]}" run -p vmnl-smoke-tests)
            ;;
        unit)
            suite_name='UNIT'
            suite_scope='workspace libraries'
            suite_kind='tests'
            test_command=(cargo "${color_args[@]}" test --workspace --lib --exclude vmnl-api-tests --exclude vmnl-gpu-tests --exclude vmnl-platform-tests --exclude vmnl-smoke-tests)
            ;;
        *)
            printf 'unknown VMNL test suite: %s\n' '{{ suite }}' >&2
            exit 2
            ;;
    esac

    passed=0
    failed=0
    ignored=0
    log_file="$(mktemp "${TMPDIR:-/tmp}/vmnl-test.XXXXXX")"
    trap 'rm -f -- "$log_file"' EXIT
    started=$SECONDS

    printf '\n%s╭─ VMNL • %s • %s%s\n' "$cyan" "$suite_name" "$suite_scope" "$reset"
    printf '%s│%s  %s\n' "$cyan" "$reset" "${test_command[*]}"
    "${test_command[@]}" 2>&1 | tee "$log_file" | sed "s/^/${cyan}│${reset}  /"
    pipeline_status=("${PIPESTATUS[@]}")
    status=0
    for command_status in "${pipeline_status[@]}"; do
        if (( command_status != 0 )); then
            status=$command_status
            break
        fi
    done
    duration="$((SECONDS - started))s"

    case "$suite_kind" in
        compile)
            detail="$(awk '/^[[:space:]]*Executable / { count += 1 } END { print count + 0 }' "$log_file") executables compiled"
            status_label='COMPILED'
            success_color=$yellow
            ;;
        smoke)
            detail='executable exited 0'
            status_label='PASS'
            success_color=$green
            ;;
        tests)
            read -r passed failed ignored < <(
                awk '
                    /test result:/ {
                        for (field = 1; field <= NF; field += 1) {
                            if ($field == "passed;") {
                                passed += $(field - 1)
                            } else if ($field == "failed;") {
                                failed += $(field - 1)
                            } else if ($field == "ignored;") {
                                ignored += $(field - 1)
                            }
                        }
                    }
                    END { printf "%d %d %d\n", passed, failed, ignored }
                ' "$log_file"
            )
            detail="$passed passed · $failed failed · $ignored ignored"
            status_label='PASS'
            success_color=$green
            ;;
    esac

    if [[ -n ${VMNL_TEST_REPORT_FILE:-} ]]; then
        printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' '{{ suite }}' "$suite_kind" "$status" "$passed" "$failed" "$ignored" "$duration" >> "$VMNL_TEST_REPORT_FILE"
        report_status=$?
        if (( status == 0 && report_status != 0 )); then
            status=$report_status
        fi
    fi

    if (( status == 0 )); then
        printf '%s╰─ %s✓ %s%s  %s · %s%s\n' "$cyan" "$success_color" "$status_label" "$reset" "$detail" "$duration"
    else
        printf '%s╰─ %s✗ FAIL%s  exit %s · %s\n' "$cyan" "$red" "$reset" "$status" "$duration"
    fi

    exit "$status"

# Run composed test workflows and print a final aggregate without parsing terminal output.
[no-exit-message]
_test-total report:
    #!/usr/bin/env bash
    set -uo pipefail

    if [[ -n ${NO_COLOR+x} ]]; then
        cyan=''
        green=''
        red=''
        reset=''
    elif [[ -t 1 && ${TERM:-dumb} != dumb ]]; then
        cyan=$'\033[1;36m'
        green=$'\033[1;32m'
        red=$'\033[1;31m'
        reset=$'\033[0m'
    else
        cyan=''
        green=''
        red=''
        reset=''
    fi

    case '{{ report }}' in
        test)
            report_name='TEST'
            report_unit='suites'
            recipes=(test-unit test-api test-smoke)
            ;;
        validate)
            report_name='VALIDATION'
            report_unit='steps'
            recipes=(build-workspace check-fmt check-clippy doctest docs _docs-api-check test-unit test-api test-smoke test-platform)
            ;;
        *)
            printf 'unknown VMNL aggregate report: %s\n' '{{ report }}' >&2
            exit 2
            ;;
    esac

    if ! report_file="$(mktemp "${TMPDIR:-/tmp}/vmnl-test-report.XXXXXX")"; then
        printf 'cannot create VMNL aggregate report\n' >&2
        exit 1
    fi
    trap 'rm -f -- "$report_file"' EXIT
    started=$SECONDS
    status=0
    executed=0
    stage_statuses=()
    stage_durations=()

    for recipe in "${recipes[@]}"; do
        executed=$((executed + 1))
        stage_started=$SECONDS
        VMNL_TEST_REPORT_FILE="$report_file" just "$recipe"
        recipe_status=$?
        stage_statuses+=("$recipe_status")
        stage_durations+=("$((SECONDS - stage_started))s")
        if (( recipe_status != 0 )); then
            status=$recipe_status
            break
        fi
    done

    read -r passed failed ignored smoke < <(
        awk -F '\t' '
            {
                passed += $4
                failed += $5
                ignored += $6
                if ($2 == "smoke" && $3 == 0) {
                    smoke += 1
                }
            }
            END { printf "%d %d %d %d\n", passed, failed, ignored, smoke }
        ' "$report_file"
    )
    duration="$((SECONDS - started))s"

    printf '\n%s╭─ VMNL %s • DETAIL%s\n' "$cyan" "$report_name" "$reset"
    for index in "${!recipes[@]}"; do
        recipe=${recipes[index]}
        record_suite=''
        case "$recipe" in
            build-workspace)
                stage_name='BUILD'
                stage_detail='workspace targets built'
                ;;
            check-clippy)
                stage_name='CLIPPY'
                stage_detail='warnings denied'
                ;;
            check-fmt)
                stage_name='FORMAT'
                stage_detail='formatting checked'
                ;;
            docs)
                stage_name='DOCS'
                stage_detail='Rustdoc built'
                ;;
            _docs-api-check)
                stage_name='DOCS-API'
                stage_detail='mdBook, inventory, snippets, and links checked'
                ;;
            doctest)
                stage_name='RUSTDOC'
                record_suite='doctest'
                ;;
            test-api)
                stage_name='API'
                record_suite='api'
                ;;
            test-smoke)
                stage_name='SMOKE'
                record_suite='smoke'
                ;;
            test-unit)
                stage_name='UNIT'
                record_suite='unit'
                ;;
            test-platform)
                stage_name='PLATFORM'
                record_suite='platform'
                ;;
        esac

        if (( index >= executed )); then
            printf '%s│%s  · %-10s not run\n' "$cyan" "$reset" "$stage_name"
            continue
        fi

        stage_status=${stage_statuses[index]}
        stage_duration=${stage_durations[index]}
        if [[ -n $record_suite ]]; then
            record="$(awk -F '\t' -v suite="$record_suite" '$1 == suite { print; exit }' "$report_file")"
            if [[ -n $record ]]; then
                IFS=$'\t' read -r _ record_kind _ record_passed record_failed record_ignored record_duration <<< "$record"
                case "$record_kind" in
                    smoke)
                        stage_detail='executable exited 0'
                        ;;
                    tests)
                        stage_detail="$record_passed passed · $record_failed failed · $record_ignored ignored"
                        ;;
                esac
                stage_duration=$record_duration
            else
                stage_detail='suite report unavailable'
            fi
        fi

        if (( stage_status == 0 )); then
            printf '%s│%s  %s✓%s %-10s %s · %s\n' "$cyan" "$reset" "$green" "$reset" "$stage_name" "$stage_detail" "$stage_duration"
        else
            printf '%s│%s  %s✗%s %-10s exit %s · %s · %s\n' "$cyan" "$reset" "$red" "$reset" "$stage_name" "$stage_status" "$stage_detail" "$stage_duration"
        fi
    done

    if (( status == 0 )); then
        printf '%s╰═ VMNL %s %s✓ PASS%s  %s/%s %s · %s passed · %s smoke · %s failed · %s ignored · %s%s\n' "$cyan" "$report_name" "$green" "$reset" "$executed" "${#recipes[@]}" "$report_unit" "$passed" "$smoke" "$failed" "$ignored" "$duration" "$reset"
    else
        printf '%s╰═ VMNL %s %s✗ FAIL%s  %s/%s %s · exit %s · %s passed · %s smoke · %s failed · %s ignored · %s%s\n' "$cyan" "$report_name" "$red" "$reset" "$executed" "${#recipes[@]}" "$report_unit" "$status" "$passed" "$smoke" "$failed" "$ignored" "$duration" "$reset"
    fi

    exit "$status"

# Run non-mutating checks.
check: check-fmt check-clippy

# Check formatting.
check-fmt:
    cargo fmt --all --check

# Run Clippy with warnings denied.
check-clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Apply formatting and automatic fixes.
lint:
    cargo fmt --all
    cargo fix --workspace --all-targets --all-features --allow-dirty --allow-staged
    cargo clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
    cargo fmt --all

# Build Rustdoc with warnings denied.
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Check the complete API book without changing tracked files.
[no-exit-message]
docs-api-check:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just docs
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _docs-api-check

# Install pinned API documentation tools under target/.
[no-exit-message]
docs-api-tools:
    ./tools/api_docs_tools.sh

# Internal API book checks; Rustdoc must already have been built.
[no-exit-message]
_docs-api-check:
    #!/usr/bin/env bash
    set -euo pipefail
    just docs-api-tools
    export PATH="$PWD/target/api-tools/bin:$PATH"
    [[ $(just --version) == 'just 1.57.0' ]]
    [[ $(mdbook --version) == 'mdbook v0.5.4' ]]
    [[ $(cargo public-api --version) == *'0.52.0'* ]]
    [[ $(lychee --version) == *'0.24.2'* ]]
    rustc +nightly-2026-03-12 --version >/dev/null
    cargo build -p vmnl
    python3 -m unittest discover -s tools/tests
    api_lib_dir="$PWD/target/api-book-libs"
    api_deps_dir="$PWD/target/debug/deps"
    mkdir -p "$api_lib_dir"
    for artifact in "$api_deps_dir"/*; do
        artifact_name=${artifact##*/}
        case "$artifact_name" in
            libvmnl-[0-9a-f]*.rlib|libvmnl-[0-9a-f]*.rmeta) continue ;;
        esac
        ln -sf "$artifact" "$api_lib_dir/$artifact_name"
    done
    ln -sf "$PWD/target/debug/libvmnl.rlib" "$api_lib_dir/libvmnl.rlib"
    CARGO_MANIFEST_DIR="$PWD/examples/raw/triangle" mdbook test docs/api -L "$api_lib_dir"
    mdbook build docs/api
    python3 tools/api_docs.py check
    mapfile -t markdown_files < <(rg --files docs -g '*.md')
    lychee --offline --include-fragments=full "${markdown_files[@]}" CONTRIBUTING.md CHANGELOG.md README.md

# Regenerate the reviewed public API snapshot and indexes.
[no-exit-message]
docs-api-update:
    #!/usr/bin/env bash
    set -euo pipefail
    just docs-api-tools
    export PATH="$PWD/target/api-tools/bin:$PATH"
    [[ $(cargo public-api --version) == *'0.52.0'* ]]
    rustc +nightly-2026-03-12 --version >/dev/null
    python3 tools/api_docs.py update

# Run the complete non-GPU validation sequence.
[no-exit-message]
validate:
    @JUST_TEMPDIR="${TMPDIR:-/tmp}" just _test-total validate

# Install Linux system dependencies.
bootstrap:
    ./deps
