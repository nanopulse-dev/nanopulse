# build all binaries (gateway + server)
build:
  cd nanopulse-gateway && just build
  cd nanopulse-server && just build

# build and generate distributable binaries (gateway + server)
dist:
  cd nanopulse-gateway && just dist
  cd nanopulse-server && just dist

# run all tests
test:
  cd nanopulse && just test
  cd nanopulse-device && just test
  cd nanopulse-gateway && just test
  cd nanopulse-hal-sx1302 && just test
  cd nanopulse-server && just test
  cd nanopulse-structs && just test

# test examples
test-examples:
  cd examples/nanopulse-device/rp-pico && just test
  cd examples/nanopulse-device/rp-pico2w && just test
