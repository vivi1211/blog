#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// 首先通过 frame_support::pallet 宏创建 pallet
#[frame_support::pallet]
pub mod pallet {
    // 引入需要的包
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    //创建配置接口，通过 config 宏完成
    //继承自系统模块的 Config 接口，只有一个
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // 只有一个关联类型就是 Event，并且约束
        // 可以从本模块的Event 类型进行转换，并且它的类型是一个系统模块的Event 类型。
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    // 定义一个结构体类型，来承载整个功能模块，使用 pallet::pallet 这个宏进行定义
    #[pallet::pallet]
    // 表示这个模块依赖的存储单元，一级存储单元依赖的 trait
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    // 通过 storage 宏来定义存储类型，用来存储存证
    #[pallet::storage]
    // 这里定义的getter方法可以通过前段接口进行调用 my_proofs 方法来查询连上的状态，也就是说没必要单独写一个读取的接口。
    #[pallet::getter(fn my_proofs)]
    pub type Proofs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>, // 存证的哈希值
        (T::AccountId, T::BlockNumber) // 值时两个元素的tuple，第一个是AccountId, 第二个存储区块高度。
    >;

    // 通过 Event 定义一个时间存储类型，这是一个枚举。
    #[pallet::event]
    // 生成一个 转换后的 metadata 方便前段接收
    #[pallet::metadata(T::AccountId = "AccountId")]
    // 生成一个帮助性的方法，方便这个方法进行触发。
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClaimCreated(T::AccountId, Vec<u8>),
        ClaimRevoked(T::AccountId, Vec<u8>),
    }

    // 通过 error 宏定义一个错误信息
    #[pallet::error]
    pub enum Error<T> {
        // 定义一个错误信息，存证已经存在
        ProofAlreadyExist,
        ClaimNotExist,
        NotClaimOwner,
    }

    // 定义一个 hooks ，如果有初始化区块的信息可以放到这里面，如果没有这个也必须要加上否则会报错
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // 构建可调用函数，通过 call 这个宏
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_claim(
            origin: OriginFor<T>,   // 这个参数表示交易的发送方
            claim: Vec<u8>, // 表示存证的哈希值
        ) -> DispatchResultWithPostInfo { // 返回值是一个Result类型的别名它会包含一些weight的信息，这是通过use引入进来的
            // TODO:: 写入创建存证的逻辑。
            // 验证签名信息是否合法
            let sender = ensure_signed(origin)?;
            // 判断存证信息是否存在
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);
            // 插入存证
            Proofs::<T>::insert(
                &claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number()),
            );

            // 发送事件
            Self::deposit_event(Event::ClaimCreated(sender, claim));
            // 返回结果信息，并进行类型转换。
            Ok(().into())
        }

        // 建立一个吊销存证的方法
        #[pallet::weight(0)]
        pub fn revoke_claim(
            origin: OriginFor<T>,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // 先验证 origin
            let sender = ensure_signed(origin)?;
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            // 判断发送者和存证所有者是否是同一个人
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            // 删除存证
            Proofs::<T>::remove(&claim);
            // 发送存证Revoked事件
            Self::deposit_event(Event::ClaimRevoked(sender, claim));
            // 返回函数成功结果
            Ok(().into())
        }

        // 建立一个转移存证的方法
        #[pallet::weight(0)]
        pub fn transfer_claim(
            from: OriginFor<T>,
            to:T::AccountId,
            claim: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            // 先验证from
            let sender = ensure_signed(from)?;
            let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
            // 判断发送者和存证所有者是否是同一个人
            ensure!(owner == sender, Error::<T>::NotClaimOwner);
            // 转移存证
            Proofs::<T>::insert(
                &claim,
                (to.clone(),frame_system::Pallet::<T>::block_number())
            );
            // 发送存证RCreated事件
            Self::deposit_event(Event::ClaimCreated(to, claim));
            // 返回函数成功结果
            Ok(().into())
        }
    }
}