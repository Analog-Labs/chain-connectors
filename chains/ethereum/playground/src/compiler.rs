use std::path::Path;
use ethers_solc::{Project, ProjectPathsConfig, SolcConfig, EvmVersion, artifacts::{Optimizer, OptimizerDetails, YulDetails}, ConfigurableArtifacts};

// #[allow(clippy::unwrap_used)]
pub fn compile(dirname: &Path) -> eyre::Result<()> {
    let paths = ProjectPathsConfig::builder().sources(dirname).build()?;

    // Solidity compiler settings
    let mut solc_config = SolcConfig::builder().build();
    solc_config.settings.metadata = None;
    solc_config.settings.remappings = Vec::with_capacity(0);
    solc_config.settings.evm_version = Some(EvmVersion::Berlin);
    solc_config.settings.optimizer = Optimizer {
        enabled: Some(true),
        runs: Some(1000),
        details: Some(OptimizerDetails {
            peephole: Some(true),
            inliner: Some(true),
            jumpdest_remover: Some(true),
            order_literals: Some(true),
            deduplicate: Some(true),
            cse: Some(true),
            constant_optimizer: Some(true),
            yul: Some(true),
            yul_details: Some(YulDetails {
                stack_allocation: Some(true),
                optimizer_steps: None,
            }),
        })
    };

    // Configure the project with all its paths, solc, cache etc.
    let project = Project::builder()
        .solc_config(solc_config)
        .paths(paths)
        .ephemeral()
        .no_artifacts()
        .build()?;
    
    // Confile the project
    let output = project.compile()?;
    return Ok(());

    for (id, artifact) in output.compiled_artifacts().artifacts::<ConfigurableArtifacts>() {
        println!("\n{}", id.identifier());

        if let Some(bytes) = artifact.bytecode.as_ref().map(|bytecode| bytecode.clone().object) {
            println!("bytecode: {bytes:?}");
        }
    }

    Ok(())
}
// 0x608060405234801561001057600080fd5b5060c68061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80633fa4f24514603757806355241077146053575b600080fd5b603d607e565b6040518082815260200191505060405180910390f35b607c60048036036020811015606757600080fd5b81019080803590602001909291905050506087565b005b60008054905090565b806000819055505056fea265627a7a723158209b607a766a9486da8fcc06a7966ee6dff7bbcc4ec2b30a30213214eefff2f44364736f6c63430005110032
// 0x608060405234801561000f575f80fd5b506101438061001d5f395ff3fe608060405234801561000f575f80fd5b5060043610610034575f3560e01c80633fa4f245146100385780635524107714610056575b5f80fd5b610040610072565b60405161004d919061009b565b60405180910390f35b610070600480360381019061006b91906100e2565b61007a565b005b5f8054905090565b805f8190555050565b5f819050919050565b61009581610083565b82525050565b5f6020820190506100ae5f83018461008c565b92915050565b5f80fd5b6100c181610083565b81146100cb575f80fd5b50565b5f813590506100dc816100b8565b92915050565b5f602082840312156100f7576100f66100b4565b5b5f610104848285016100ce565b9150509291505056fea2646970667358221220457ec1632ab7440f507c91944cbe7c552ae70eca4af7799552d11c077e98168c64736f6c63430008160033
