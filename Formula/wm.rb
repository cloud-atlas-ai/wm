class Wm < Formula
  desc "Working memory for AI coding assistants"
  homepage "https://github.com/cloud-atlas-ai/wm"
  url "https://github.com/cloud-atlas-ai/wm/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "wm", shell_output("#{bin}/wm --help")
  end
end
