



use super::{
    VariableNodeContext, VariableConfig, GetVariableConfig,BacktestNodeContextTrait,
    Response
};
use tokio::sync::oneshot;
use event_center::{
    communication::backtest_strategy::{
        GetCustomVariableValueCmdPayload, GetCustomVariableValueCommand, UpdateCustomVariableValueCmdPayload, UpdateCustomVariableValueCommand
    }, 
    event::node_event::backtest_node_event::{
        variable_node_event::{CustomVariableUpdateEvent, CustomVariableUpdatePayload}, 
        VariableNodeEvent
    }
};




impl VariableNodeContext {
    
    pub(super) async fn handle_condition_trigger_variable(&mut self, condition_trigger_configs: &Vec<VariableConfig>) {
        tracing::debug!("handle_condition_trigger_variable: {:#?}", condition_trigger_configs);
        
        // 分成get, update, reset三批
        let get_var_configs = condition_trigger_configs.iter().filter(|config| config.is_get()).cloned().collect::<Vec<_>>();
        let update_var_configs = condition_trigger_configs.iter().filter(|config| config.is_update()).cloned().collect::<Vec<_>>();
        let reset_var_configs = condition_trigger_configs.iter().filter(|config| config.is_reset()).cloned().collect::<Vec<_>>();

        self.get_variable(&get_var_configs).await;
        self.update_variable(&update_var_configs).await;


        

    }


    async fn get_variable(&self, get_var_configs: &Vec<VariableConfig>) {
        // 先生成Handler,然后同时执行
        let mut get_handles = Vec::new();
        let node_id = self.get_node_id();
        let node_name = self.get_node_name();
        let strategy_command_sender = self.get_strategy_command_sender().clone();

        for config in get_var_configs {
            if let VariableConfig::Get(GetVariableConfig::Custom(custom_config)) = config {
                let var_name = custom_config.var_name().to_string();
                let config_id = custom_config.config_id();
                let node_name = node_name.clone();
                let node_id = node_id.clone();
                let play_index = self.get_play_index();
                let output_handle_id = custom_config.output_handle_id.clone();
                let sender_clone = strategy_command_sender.clone();
                let output_handle = self.get_output_handle(&output_handle_id.clone()).clone();
                
                let handle = tokio::spawn(async move {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let get_custom_var_event = GetCustomVariableValueCmdPayload::new(var_name.clone());
                    let cmd = GetCustomVariableValueCommand::new(node_id.clone(), resp_tx, Some(get_custom_var_event));
                    sender_clone.send(cmd.into()).await.unwrap();
                    let response = resp_rx.await.unwrap();
                    if response.is_success() {
                        let payload = CustomVariableUpdatePayload::new(
                            play_index,
                            node_id.clone(),
                            config_id,
                            var_name,
                            response.var_value.clone()
                        );
                        let var_event: VariableNodeEvent = CustomVariableUpdateEvent::new(
                            node_id.clone(),
                            node_name,
                            output_handle_id,
                            payload
                        ).into();
                        let _ = output_handle.send(var_event.into());

                    }
                });
                get_handles.push(handle);
            }

        }
        
        // 等待所有任务完成
        futures::future::join_all(get_handles).await;

    }


    async fn update_variable(&self, update_var_configs: &Vec<VariableConfig>) {
        tracing::debug!("update_variable: {:?}", update_var_configs);
        // 先生成Handler,然后同时执行
        let mut get_handles = Vec::new();
        let node_id = self.get_node_id();
        let node_name = self.get_node_name();
        let strategy_command_sender = self.get_strategy_command_sender().clone();

        for config in update_var_configs {
            if let VariableConfig::Update(update_var_config) = config {
                
                let var_name = update_var_config.var_name().to_string();
                let config_id = update_var_config.config_id();
                let node_name = node_name.clone();
                let node_id = node_id.clone();
                let play_index = self.get_play_index();
                let output_handle_id = update_var_config.output_handle_id.clone();
                let sender_clone = strategy_command_sender.clone();
                let output_handle = self.get_output_handle(&output_handle_id.clone()).clone();

                let update_var_config_clone = update_var_config.clone();
                
                let handle = tokio::spawn(async move {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let update_var_event = UpdateCustomVariableValueCmdPayload::new(update_var_config_clone.clone());
                    let cmd = UpdateCustomVariableValueCommand::new(node_id.clone(), resp_tx, Some(update_var_event));
                    sender_clone.send(cmd.into()).await.unwrap();
                    let response = resp_rx.await.unwrap();
                    if response.is_success() {
                        tracing::debug!("update_variable success: {:?}", response.var_value);
                        let payload = CustomVariableUpdatePayload::new(
                            play_index,
                            node_id.clone(),
                            config_id,
                            var_name,
                            response.var_value.clone()
                        );
                        let var_event: VariableNodeEvent = CustomVariableUpdateEvent::new(
                            node_id.clone(),
                            node_name,
                            output_handle_id,
                            payload
                        ).into();
                        let _ = output_handle.send(var_event.into());

                    } else {
                        tracing::error!("update_variable failed: {:?}", response.get_error());
                    }
                });
                get_handles.push(handle);
            }

        }
        
        // 等待所有任务完成
        futures::future::join_all(get_handles).await;

    }
}