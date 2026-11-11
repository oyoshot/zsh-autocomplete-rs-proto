{
  lib,
  rustPlatform,
  zsh,
  shellInstallDir ? "share/zsh-autocomplete-rs",
}:

rustPlatform.buildRustPackage {
  pname = "zsh-autocomplete-rs";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./shell
      ./tests
      ./benches
    ];
  };
  cargoLock.lockFile = ./Cargo.lock;
  cargoBuildFlags = [
    "--bin"
    "zsh-autocomplete-rs"
  ];

  # Native tests need PTYs and Unix sockets; run them outside the Nix build.
  doCheck = false;
  nativeBuildInputs = [ zsh ];
  postInstall = ''
    shellDir="$out"/${lib.escapeShellArg shellInstallDir}
    install -Dm644 shell/*.zsh -t "$shellDir"
    for shellFile in "$shellDir"/*.zsh; do
      zsh -dfc 'zcompile -U "$1"' _ "$shellFile"
    done
  '';

  doInstallCheck = true;
  nativeInstallCheckInputs = [ zsh ];
  installCheckPhase = ''
    runHook preInstallCheck
    shellDir="$out"/${lib.escapeShellArg shellInstallDir}
    for shellFile in "$shellDir"/*.zsh; do
      zsh -n "$shellFile"
      zsh -dfc 'zcompile -t "$1"' _ "$shellFile.zwc"
    done
    runHook postInstallCheck
  '';

  meta = {
    description = "Zsh autocomplete with a Rust popup renderer";
    homepage = "https://github.com/oyoshot/zsh-autocomplete-rs-proto";
    mainProgram = "zsh-autocomplete-rs";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
