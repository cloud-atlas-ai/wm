class Wm < Formula
  desc "Working memory for AI coding assistants"
  homepage "https://github.com/cloud-atlas-ai/wm"
  url "https://github.com/cloud-atlas-ai/wm/archive/refs/tags/v0.1.1.tar.gz"
  sha256 "29d7228b2c538cae3d3814a36413fa7d2152ef4c09be95c2b93561e3fbe23d81"
  license :cannot_represent

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "wm", shell_output("#{bin}/wm --help")
  end
end
