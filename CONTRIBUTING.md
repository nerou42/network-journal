# Contributions are welcome!

## How to build an RPM package

1. Build the docker image  
`docker build -t network-journal/rpmbuild -f pkg/rpmbuild.dockerfile .`
1. Run the docker image interactively  
`docker run -v ./pkg/network-journal.spec:/root/rpmbuild/SPECS/network-journal.spec -v ./pkg:/root/rpmbuild/target -it network-journal/rpmbuild:latest /bin/bash`
1. Run the following commands inside the container while replacing `<version>` with the version to build (e.g. "0.1.0"):  
```bash
wget http://github.com/nerou42/network-journal/archive/network-journal-<version>/network-journal-<version>.tar.gz -O SOURCES/network-journal-<version>.tar.gz
rpmbuild -bb SPECS/network-journal.spec
mv RPMS/x86_64/network-journal-<version>-1.el9.x86_64.rpm target/
```
4. You now should have a RPM package inside your `pkg` folder.

