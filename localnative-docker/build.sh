cd `dirname $0`

set -e
docker build . -t localnative

tar -czpf localnative.tar.gz Dockerfile
