use crate::account_info::Model as AccountInfoModel;
use star_river_core::account::AccountInfo;

impl From<AccountInfoModel> for AccountInfo {
    fn from(info: AccountInfoModel) -> Self {
        AccountInfo {
            id: info.id,
            account_id: info.account_id,
            info: info.info.unwrap(),
            create_time: info.create_time,
            update_time: info.update_time,
        }
    }
}
