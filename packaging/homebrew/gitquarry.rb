class Gitquarry < Formula
  desc "Terminal CLI for public GitHub repository search with explicit discovery controls"
  homepage "https://github.com/Microck/gitquarry"
  version "0.1.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.3/gitquarry-v0.1.3-aarch64-apple-darwin.tar.gz"
      sha256 "fd96560132f65aff6f2c64de6568e0443eef8c1446eb7627febc905af9a35dde"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.3/gitquarry-v0.1.3-x86_64-apple-darwin.tar.gz"
      sha256 "b9672e6fde7be95b01d66db8a425d73d6415a27830e1f5aee4b45a7d23064ce2"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.3/gitquarry-v0.1.3-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "b12863618a2ca068ecbf05430ae239d7516a25cb251cae1b129711d7f1c1da13"
    end

    if Hardware::CPU.intel?
      url "https://github.com/Microck/gitquarry/releases/download/v0.1.3/gitquarry-v0.1.3-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9a9449e64ccf2707f5d32010dbfe5162831e89eb0041901b269a1eb179fd74d9"
    end
  end

  def install
    bin.install "gitquarry"
  end

  test do
    assert_match "Usage: gitquarry [OPTIONS] [COMMAND]", shell_output("#{bin}/gitquarry --help")
  end
end
