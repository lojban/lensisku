# Pitfalls

Mistakes made while working on lensisku, what they cost, and the rule that prevents each one. Read this
before your first change; add to it when you make a new mistake worth not repeating. Keep entries short:
what happened, then the rule. (Design rules that came from earlier bugs, such as blocking the runtime or
untyped SQL binds, are in [ARCHITECTURE-PRINCIPLES.md](ARCHITECTURE-PRINCIPLES.md).)

## Building and checking

**`cargo fmt --check` failed in CI after a local format.** The default toolchain is a nightly without
rustfmt, so formatting had only been applied to the files being edited, and three untouched files were
already unformatted.
- Format with `cargo +stable fmt`, and run `cargo +stable fmt --check` last, after every edit.

## Secrets

**A password appeared in tool output.** A `grep … | sed` that was meant to mask secret values did not match
the name `gleki_smtp_pass=`, so the value was printed once while checking a config.
- Never print the `secrets` file or an environment dump. To check a setting exists, test for the name only
  (`grep -c '^name='`), or mask the whole value (`sed -E 's/=.*/=***/'`) on every line.
- A leaked value is rotated, not just forgotten.

**The server's `secrets` file was readable by every user.** It was mode 644 and holds passwords.
- Secret files are mode 600, and so are their backups (`cp -p` keeps the old mode, so `chmod 600` the copy).

**Secrets travel into logs and images.** On the servers the systemd unit runs the container script with
`bash -x`, which writes assigned variable values (including secrets) to the journal, and the Dockerfile
templates bake secret values into the image as `ENV`.
- Do not trace scripts that handle secrets, and prefer runtime secrets (podman secrets) over `ENV` in an image.
  Both are listed as open work in the containers audit.

## Deploying

**Changing a template or `secrets` does nothing until the image is rebuilt.** The containers repo fills the
Dockerfiles from `config` and `secrets` at build time.
- After a change there: `./build_image.sh <container>`, then restart the service, and check the running
  container's environment (`podman exec <c> printenv NAME`) instead of trusting the file.

**A long deploy step died when the session was interrupted.** The command ran in the foreground over SSH.
- Run builds detached (`nohup … &`) with a log file and a status file, then poll the status file and check
  `git log` on the host. A status file that never appears means the job never started.

**The production container logs nothing.** It runs with `--log-driver=none`, so `podman logs` is empty and
an error id shown to a user (for example fe7422a3bdca) cannot be looked up.
- When a user reports an error id on production, expect to reproduce it rather than read it. Fixing this
  (a log driver or a log file) is open work.

## Mail

**Port 25 is blocked by the ISP.** Mail sent from the server stopped leaving.
- Send through port 587 with STARTTLS (the app requires it). Use the mail host's local name from inside the
  network so mail does not go out to the internet and back.
- "The port is open" is not a test. Log in with the real credentials from inside the running container, and
  confirm a notification email was actually written to `sent_emails` afterwards.

**A guessed cause was wrong.** A certificate-name mismatch was blamed for a failed password reset before it was
tested. Running the app's exact TLS and credentials settings from inside the container showed the mail
connection was fine.
- Reproduce with the real program and settings before naming a cause, and say plainly when you could not.

## Courses and import/export

**An import tool existed for one kind of card.** A separate "import dialogues" feature sat beside the generic
course import and could not carry everything a course contains (several scored questions, levels, audio).
- There is one export/import format for collections (`lensisku.collection`, see
  [file-formats.md](file-formats.md)). A new kind of card is added to that format, never to a side importer.

**Scores and progress leaked into the design of a course file.** Per-question scoring of dialogues is a rule of
the learner's progress, not of the course.
- Export only the schema and the content. The dialogue script is the single source of truth for lines and
  questions; derived structure is read-only and ignored on import. Test a round trip with a dialogue that has
  several questions.
