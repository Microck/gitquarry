class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.2.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.1/gitquarry-v0.2.1-aarch64-apple-darwin.tar.gz"
      sha256 "b02898093c43a7ee08a86b1f5b491dc172d70159add93c798e930f5d21948663"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.1/gitquarry-v0.2.1-x86_64-apple-darwin.tar.gz"
      sha256 "2caa1b6bb8da43c82c6d51a8be8ce3964f4abff73ca6044f884b85d7d422087e"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.1/gitquarry-v0.2.1-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "e8db7a0698c9651e46d565c2187a7a37960f8a409a7fba538a279eb19eb5b0e5"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.2.1/gitquarry-v0.2.1-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "f980c6f2087d5978a6d33144c10226dc8959e45c3660bdfa49f6e17329b18bf0"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
