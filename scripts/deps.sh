if [ -z ${TRAVIS_OS_NAME+x} ]; then
    case `uname -s` in
        Linux)
            TRAVIS_OS_NAME=linux
            ;;
        Darwin)
            TRAVIS_OS_NAME=osx
            ;;
        *)
            echo "Unknown OS"
            exit 1
    esac
fi

case $TRAVIS_OS_NAME in
    linux)
        pyenv version 3
        PIP=pip
        PYTHON=python
        ;;
    osx)
        if ! which python3 > /dev/null; then
            brew install python3 || brew upgrade python
        fi
        PIP=pip3
        PYTHON=python3
        ;;
    local-archlinux)
        PIP=pip3
        PYTHON=python3
        ;;
esac

$PIP install --quiet --user sh toml
