// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL1DAValidator, L1DAValidatorOutput} from "./IL1DAValidator.sol";

contract BitcoinDAL1DAValidator is IL1DAValidator {
    function checkDA(
        uint256, // _chainId
        bytes32, // _l2DAValidatorOutputHash
        bytes calldata _operatorDAInput,
        uint256 maxBlobsSupported
    ) external override returns (L1DAValidatorOutput memory output) {
        // For BitcoinDA, we expect the operator to just provide the data for us.
        // We don't need to do any checks with regard to the l2DAValidatorOutputHash.
        require(_operatorDAInput.length == 32);

        bytes32 stateDiffHash = abi.decode(_operatorDAInput, (bytes32));
        _verifyBitcoinDA(stateDiffHash);
        // The rest of the fields that relate to blobs are empty.
        output.stateDiffHash = stateDiffHash;

        output.blobsLinearHashes = new bytes32[](maxBlobsSupported);
        output.blobsOpeningCommitments = new bytes32[](maxBlobsSupported);
    }

    function supportsInterface(bytes4 interfaceId) external pure returns (bool) {
        return interfaceId == type(IL1DAValidator).interfaceId;
    }

    function _verifyBitcoinDA(
        bytes32 _dataHash
    ) internal view {
        address BITCOINDA_PRECOMPILE_ADDRESS = address(0x63);
        uint16 BITCOINDA_PRECOMPILE_COST = 1400;
        (bool success, bytes memory result) = BITCOINDA_PRECOMPILE_ADDRESS.staticcall{gas: BITCOINDA_PRECOMPILE_COST}(abi.encode(_dataHash));

        require(success, "Staticcall failed.");
        require(result.length > 0, "Return data must not be empty.");
        
    }
}
