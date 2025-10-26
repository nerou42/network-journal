FROM almalinux:9-minimal

# microdnf install -y almalinux-release-devel && microdnf install -y python3-rust2rpm
RUN rpm -e --nodeps coreutils-single && \
    microdnf upgrade -y && \
    microdnf install -y wget tar git chkconfig coreutils diffutils patch gcc openssl-devel && \
    microdnf install -y rpm-build rpm-devel rpmlint rpmdevtools && \
    (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y) && \
	microdnf clean all && rm -rf /var/cache/yum && \
    mkdir /root/rpmbuild && cd /root/rpmbuild && \
    rpmdev-setuptree

WORKDIR /root/rpmbuild
CMD ["/bin/sh"]
