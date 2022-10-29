# desktop
Run locally is easy. No special setup required:

```
cargo run
```

# mobile
To build for mobile you have to install xbuild:

```
cargo install xbuild --git https://github.com/cloudpeers/xbuild --branch no-flutter
```

On ios a provisioning profile and key is required.

```
x run --device imd:55abbd4b70af4353bdea2595bbddcac4a2b7891a --provisioning-profile ~/analog-apple-credentials/wildcard.mobileprovision --pem ~/analog-apple-credentials/development.pem
```

On android it can be run with

```
x run --device adb:16ee50bc
```

Note that the ANDROID_HOME env variable needs to point to the android sdk.

# web

To run in the browser install the dioxus cli tool.

```
cargo install --git https://github.com/dioxuslabs/cli
```

Next you have to build the project for the first time.

```
dioxus build
```

Now that the dist folder was created you can start using hot reload:

```
dioxus serve
```
