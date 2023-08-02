#!/bin/bash
base=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "base is at $base"

for i in "$@"; do
  case $i in
    --clean)
      CLEAN=1
      shift # past argument with no value
      ;;
    -*|--*)
      echo "Unknown option $i"
      exit 1
      ;;
    *)
      ;;
  esac
done

genAbi() {
	echo generating abi...
	cd $base/caramell-blockchain/caramell-blockchain/truffle
	truffle compile &&
	cd $base
}

moveAbiToBindings() {
	echo moving abi to bindings...
	cp -r $base/caramell-blockchain/caramell-blockchain/truffle/build/contracts \
		$base/createBindings/
}

moveBindingsToDest() {
	echo moving binding to dest
	cp -r $base/createBindings/src/contracts $base/utils/src/
}

clean() {
	echo cleaning...
	rm -rf $base/createBindings/contracts \
		$base/caramell-blockchain/caramell-blockchain/w3rs/src/contracts \
		$base/caramell-blockchain/caramell-blockchain/truffle/build/ \
		$base/utils/src/contracts
}


# clean everything, regen fresh abi & prep for bindings
([ $CLEAN ] && clean || [ 1 ]) && genAbi && moveAbiToBindings

# gen bindings & put them into dest
cargo run --bin createBindings && (moveBindingsToDest && echo "all done, copied to $base/utils/src/") || echo Failure !
