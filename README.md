# NGAKALIN

Ngakalin is tool for building mock api inspired by [WireMock](https://wiremock.org/)

**How to run**
```SHELL
git clone git@github.com:hexennacht/ngakalin.git

cargo build --release

./target/release/ngakalin -c ./configuration.yaml
```


**TODO List**
* [ ] Scheduler to register new endpoint
* [ ] Add tracing and logging
* [ ] Add dockerfile