## Concordium Mulitisig contract
This smart contract depicts a multi-signatory functionality on Concordium blockchain where a minimum of three account holders must sign/permit a transaction out of the contract but anyone can send tokens into it.


### ScreenShots  
* Deploying the Module
![Screenshot (208)](https://github.com/josidbobo/concordium-multisig/assets/38986781/75cd4a06-3fbe-40ca-bed6-c50cd61da40b)

* Initialising the Contract init function
  I tried by all means to init the contract but it kept returning this error _JSON could'nt be used because no schema for it_. I've followed the instructions from the documentation on schema -> https://developer.concordium.software/en/mainnet/smart-contracts/guides/build-schema.html and initialising contracts as well as asked on the [support](https://support.concordium.software) but the error persists. I ensured to use the _--schema-embed --schema-json-out and --schema-base64-out_ tags when building the module and followed @pablozsc"s instructions on the issue but the error remains unresolved. I've tried and tried again.
  ![Screenshot (209)](https://github.com/josidbobo/concordium-multisig/assets/38986781/500e5a31-67a1-4768-bddb-49c3fa75d18b)



