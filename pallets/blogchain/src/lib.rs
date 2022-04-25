#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Hash,  //hash_of计算哈希使用
		traits::{Currency,ExistenceRequirement},
		inherent::Vec,
		transactional};
	use frame_system::pallet_prelude::*;

	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency:Currency<Self::AccountId>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//定义一些常量用来限制博文内容的长度
		#[pallet::constant]
        type BlogPostMinBytes: Get<u32>;// <-- new

        #[pallet::constant]
        type BlogPostMaxBytes: Get<u32>;// <-- new

        #[pallet::constant]
        type BlogPostCommentMinBytes: Get<u32>;// <-- new

        #[pallet::constant]
        type BlogPostCommentMaxBytes: Get<u32>; // <-- new
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info] 
	//#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct BlogPost<T: Config> {
			pub content: Vec<u8>,
			pub author: <T as frame_system::Config>::AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct BlogPostComment<T: Config> {
			pub content: Vec<u8>,
			pub blog_post_id: T::Hash,
			pub author: <T as frame_system::Config>::AccountId,
	}

	
	#[pallet::storage]
	#[pallet::getter(fn blog_posts)]
	//T后面要加上T:Config
	pub type BlogPosts<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,  //以博客的id为key
		BlogPost<T>,  //博客内容为value
		>;

	#[pallet::storage]
	#[pallet::getter(fn blog_post_comments)]
	pub type BlogPostComments<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,  //以博客的id为key
		Vec<BlogPostComment<T>>,  //博客评论内容为value，一条博客的评论是个集合
		>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//创建一个博文参数：博文内容，作者，id
		BlogPostCreated(Vec<u8>,T::AccountId,T::Hash),
		BlogPostCommentCreated(Vec<u8>,T::AccountId,T::Hash),
		Tipped(T::AccountId,T::Hash),//打赏
	}

	
	#[pallet::error]
	pub enum Error<T> {
		BlogPostNotEnoughBytes, // <-- new
        BlogPostTooManyBytes, // <-- new
        BlogPostCommentNotEnoughBytes,// <-- new
        BlogPostCommentTooManyBytes,// <-- new
        BlogPostNotFound,// <-- new
        TipperIsAuthor,// <-- new
	}

	
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[transactional]	//这个宏使得这个外部交易的状态变化只有在它没有返回错误时才进行
			//因为我们同时添加了新的博文和一个空的评论列表，只有所有变化都通过时才持久化这个变化
		pub fn create_blog_post(origin: OriginFor<T>, content: Vec<u8>) -> DispatchResult {
			
			let author = ensure_signed(origin)?;
			//检查博文长度
			ensure!(
				//将content.len()转换为u32,否则类型不匹配
				(content.len() as u32)>T::BlogPostMinBytes::get(),
				<Error<T>>::BlogPostNotEnoughBytes
			);
			ensure!(
				(content.len() as u32)<T::BlogPostMaxBytes::get(),
				<Error<T>>::BlogPostTooManyBytes
			);
			//获取博文结构体{内容，作者}
			let blog_post=BlogPost{content:content.clone(),author:author.clone()};
			//获取博文id,对结构体的哈希
			let blog_post_id=T::Hashing::hash_of(&blog_post);
			BlogPosts::<T>::insert(blog_post_id,blog_post);

			//获取属于这个博文id的评论集合
			let comment_vec:Vec<BlogPostComment<T>>=Vec::new();
			BlogPostComments::<T>::insert(blog_post_id,comment_vec);
			//触发事件
			Self::deposit_event(Event::BlogPostCreated(content,author,blog_post_id));
			Ok(())
		}

	
		#[pallet::weight(0)]
		pub fn create_blog_post_comment(
			origin: OriginFor<T>, 
			content: Vec<u8>,
			blog_post_id:T::Hash,) -> DispatchResult {
			
			let author = ensure_signed(origin)?;
			//检查评论长度
			ensure!(
				(content.len() as u32)>T::BlogPostCommentMinBytes::get(),
				<Error<T>>::BlogPostCommentNotEnoughBytes
			);
			ensure!(
				(content.len() as u32)<T::BlogPostCommentMaxBytes::get(),
				<Error<T>>::BlogPostCommentTooManyBytes
			);
			//创建博文评论结构体{内容，id,作者}
			let blog_post_comment=BlogPostComment{
				content:content.clone(),
				blog_post_id:blog_post_id.clone(),
				author:author.clone(),
			};
			//添加评论，不是在原有的staragemap中添加（insert)值,而是操作现有的条目
			//原来的value是一个评论集合，在这个集合里新新增
			<BlogPostComments<T>>::mutate(blog_post_id,|comments| match comments{
				None=>Err(()),
				Some(vec)=>{
					//更新vec的方法，push
					vec.push(blog_post_comment);
					Ok(())
				}
			}
			).map_err(|_| <Error<T>>::BlogPostNotFound)?;
			
			//触发事件
			Self::deposit_event(Event::BlogPostCommentCreated(content,author,blog_post_id));
			Ok(())
		}
		
		#[pallet::weight(0)]
		pub fn tip_blog_post(
			origin: OriginFor<T>, 
			blog_post_id:T::Hash,
			amount:<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,) 
			-> DispatchResult {
				let tipper=ensure_signed(origin)?;
				//调用blog_posts getter函数获取博文内容
				let blog_post=Self::blog_posts(&blog_post_id).ok_or(<Error<T>>::BlogPostNotFound)?;
				let blog_post_author=blog_post.author;
				//确定打赏的人不是作者本人
				ensure!(blog_post_author!=tipper,<Error<T>>::TipperIsAuthor);
				//转账
				T::Currency::transfer(
					&tipper,
					&blog_post_author,
					amount,
					ExistenceRequirement::KeepAlive,
				)?;

				Self::deposit_event(Event::Tipped(tipper,blog_post_id));
				Ok(())	
			}
	}
}
