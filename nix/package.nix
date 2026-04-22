{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "gitquarry";
  version = "0.1.3";

  src = lib.cleanSource ../.;
  cargoLock.lockFile = ../Cargo.lock;

  meta = with lib; {
    description = "Terminal CLI for GitHub repository search with explicit discovery controls";
    homepage = "https://github.com/Microck/gitquarry";
    license = licenses.mit;
    mainProgram = "gitquarry";
    platforms = platforms.unix;
  };
}
