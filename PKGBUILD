# Maintainer: carlislepadolina-tech <your-email@example.com>
pkgname=apm-git
pkgver=0.1.0
pkgrel=1
pkgdesc="A lightweight AUR helper built in Rust"
arch=('x86_64')
url="https://github.com/carlislepadolina-tech/apm"
license=('MIT')
depends=('git' 'base-devel')
makedepends=('rust' 'cargo')
provides=('apm')
conflicts=('apm')
source=("git+https://github.com/carlislepadolina-tech/apm.git#branch=main")
sha256sums=('SKIP')

build() {
  cd "$srcdir/apm"
  cargo build --release --locked
}

package() {
  cd "$srcdir/apm"
  install -Dm755 "target/release/apm" "$pkgdir/usr/bin/apm"
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
