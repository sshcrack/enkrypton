# [Choice] Debian OS version (use bookworm, or bullseye on local arm64/Apple Silicon): bookworm, buster, bullseye
ARG VARIANT="bookworm"
FROM debian:${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    # Remove imagemagick due to https://security-tracker.debian.org/tracker/CVE-2019-10131
    && apt-get purge -y imagemagick imagemagick-6-common

# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive && \
apt-get -y install --no-install-recommends libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    jq \
    ca-certificates


WORKDIR /tmp
RUN wget -O latest https://api.github.com/repos/PowerShell/PowerShell/releases/latest && \
 wget -O powershell.deb $(cat latest | jq -r ".assets[] | .browser_download_url |  select(. | contains(\".deb\")) | select(. | contains(\"-lts_\") | not)") && \
 dpkg -i powershell.deb && \
 rm powershell.deb