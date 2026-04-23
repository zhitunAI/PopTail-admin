use crate::models::CasbinInfo;
use casbin::{CoreApi, DefaultModel, Enforcer, MemoryAdapter, MgmtApi};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CasbinPort {
    policies_by_role: HashMap<u32, Vec<CasbinInfo>>,
}

impl CasbinPort {
    pub async fn bootstrap(seed: HashMap<u32, Vec<CasbinInfo>>) -> Result<Self, String> {
        let model = DefaultModel::from_str(
            r#"
            [request_definition]
            r = sub, obj, act

            [policy_definition]
            p = sub, obj, act

            [policy_effect]
            e = some(where (p.eft == allow))

            [matchers]
            m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
        "#,
        )
        .await
        .map_err(|err| err.to_string())?;
        let adapter = MemoryAdapter::default();
        let mut enforcer = Enforcer::new(model, adapter)
            .await
            .map_err(|err| err.to_string())?;

        for (authority_id, policies) in &seed {
            let subject = authority_id.to_string();
            for policy in policies {
                enforcer
                    .add_policy(vec![
                        subject.clone(),
                        policy.path.clone(),
                        policy.method.clone(),
                    ])
                    .await
                    .map_err(|err| err.to_string())?;
            }
        }

        let mut baked = HashMap::new();
        for authority_id in seed.keys() {
            let subject = authority_id.to_string();
            let lines = enforcer.get_filtered_policy(0, vec![subject]);
            let mut out = Vec::new();
            for line in lines {
                if line.len() >= 3 {
                    out.push(CasbinInfo {
                        path: line[1].clone(),
                        method: line[2].clone(),
                    });
                }
            }
            baked.insert(*authority_id, out);
        }

        Ok(Self {
            policies_by_role: baked,
        })
    }

    pub fn get_policy_path_by_authority_id(&self, authority_id: u32) -> Vec<CasbinInfo> {
        self.policies_by_role
            .get(&authority_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_policy_path_by_authority_id(
        &mut self,
        authority_id: u32,
        policies: Vec<CasbinInfo>,
    ) {
        self.policies_by_role.insert(authority_id, policies);
    }

    pub fn is_allowed(&self, authority_id: u32, path: &str, method: &str) -> bool {
        self.policies_by_role
            .get(&authority_id)
            .is_some_and(|items| items.iter().any(|p| p.path == path && p.method == method))
    }
}
