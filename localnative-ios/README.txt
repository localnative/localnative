- build ios lib
```shell
cargo install cargo-lipo
rustup target add aarch64-apple-ios x86_64-apple-ios
../script/build-ios.sh 
```
- build project
```shell
install cocoapods
```

- build error (Mac M1): "zsh: abort pod install"
```shell
sudo arch -x86_64 gem install ffi
arch -x86_64 pod install
```

- build error: No such module 'UITextView_Placeholder' <br>
open project using localnative-ios.xcworkspace instead of localnative-ios.xcodeproj
