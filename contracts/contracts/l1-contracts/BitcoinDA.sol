// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL1DAValidator, L1DAValidatorOutput} from "./IL1DAValidator.sol";

contract BitcoinDAL1DAValidator is IL1DAValidator {
    function checkDA(
        uint256, // _chainId
        bytes32 _l2DAValidatorOutputHash,
        bytes calldata _operatorDAInput,
        uint256 maxBlobsSupported
    ) external override returns (L1DAValidatorOutput memory output) {
        {
            (
               bytes32 uncompressedStateDiffHash,
               bytes32 fullPubdataHash,
            ) = _processL2RollupDAValidatorOutputHash(_l2DAValidatorOutputHash, _operatorDAInput);
            // circuit will commit to the uncompressed diff hash but we related the compressed to the uncompressed one via validatePubData and recomputed the hash to check here. 
            output.stateDiffHash = uncompressedStateDiffHash;
            // fullPubdataHash should include user_logs, l2_to_l1_messages, published_bytecodes, compressed state_diffs
            _verifyBitcoinDA(fullPubdataHash);
        }
        // The rest of the fields that relate to blobs are empty.
        output.blobsLinearHashes = new bytes32[](maxBlobsSupported);
        output.blobsOpeningCommitments = new bytes32[](maxBlobsSupported);
    }

    function _processL2RollupDAValidatorOutputHash(
        bytes32 _l2DAValidatorOutputHash,
        bytes calldata _operatorDAInput
    )
        internal
        pure
        returns (
            bytes32 uncompressedStateDiffHash,
            bytes32 fullPubdataHash
        )
    {
        // Ensure the operator DA input is at least 32 bytes long
        require(_operatorDAInput.length >= 32, "Operator DA input too small");
        // we are isolating fullPubdataHash to ensure its correctness here by checking state diff(validated) + full data hash(unvalidated) == l2 DA validator output(validated)
        // fullPubdataHash is used as DA and commits to the compressed state diff (zk circuit input)
        uncompressedStateDiffHash = bytes32(_operatorDAInput[:32]);
        fullPubdataHash = bytes32(_operatorDAInput[32:64]);

        // Now, we need to double check that the provided input was indeed returned by the L2 DA validator.
        if (keccak256(_operatorDAInput[:64]) != _l2DAValidatorOutputHash) {
            revert InvalidL2DAOutputHash();
        }
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
