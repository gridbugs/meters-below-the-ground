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

if [[ "$TRAVIS_OS_NAME" == "linux" ]]; then
    pyenv version 3.6
    PIP=pip
    PYTHON=python
elif [[ "$TRAVIS_OS_NAME" == "osx" ]]; then
    PIP=pip3.6
    PYTHON=python3.6
elif [[ "$TRAVIS_OS_NAME" == "local-archlinux" ]]; then
    # this allows for local testing before submitting to travis-ci
    PIP=pip3.6
    PYTHON=python3.6
fi

$PIP install --quiet --user sh toml
