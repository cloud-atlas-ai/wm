class Wm < Formula
  desc "Working memory for AI coding assistants"
  homepage "https://github.com/cloud-atlas-ai/wm"
  url "https://github.com/open-horizon-labs/wm/archive/refs/tags/v0.3.1.tar.gz"
  sha256 "4464692a0f30c23da04f4fb006f2ce9de28a414548efbfe833ca60f68b30605e"
  license :cannot_represent

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "wm", shell_output("#{bin}/wm --help")
  end
end
