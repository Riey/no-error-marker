pub unsafe trait OptSend {
    const IMPL_SEND: bool;
}

pub unsafe trait OptSync {
    const IMPL_SYNC: bool;
}

unsafe impl<T: ?Sized> OptSend for T {
    default const IMPL_SEND: bool = false;
}

unsafe impl<T: Send + ?Sized> OptSend for T {
    const IMPL_SEND: bool = true;
}

unsafe impl<T: ?Sized> OptSync for T {
    default const IMPL_SYNC: bool = false;
}

unsafe impl<T: Sync + ?Sized> OptSync for T {
    const IMPL_SYNC: bool = true;
}
