use crate::types::{Direction, OrderID, Price, Size};

pub(crate) trait Order {
    fn get_order_id(&self) -> OrderID;
    fn get_order_size(&self) -> Size;
    fn mut_order_size(&mut self) -> &mut Size;
    fn get_order_direction(&self) -> Direction;
    fn extract_body(self) -> OrderInfo;
}

pub(crate) trait PricedOrder
    where Self: Order
{
    fn get_price(&self) -> Price;
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) struct OrderInfo {
    pub order_id: OrderID,
    pub size: Size,
    pub direction: Direction,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrder(OrderInfo);

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrder {
    info: OrderInfo,
    price: Price,
}

impl MarketOrder {
    pub fn new(order_id: OrderID, size: Size, direction: Direction) -> MarketOrder {
        MarketOrder(OrderInfo { order_id, size, direction })
    }
}

impl LimitOrder {
    pub fn new(order_id: OrderID, size: Size, direction: Direction, price: Price) -> LimitOrder {
        LimitOrder { info: OrderInfo { order_id, size, direction }, price }
    }
}

impl const Order for MarketOrder {
    fn get_order_id(&self) -> OrderID { self.0.order_id }
    fn get_order_size(&self) -> Size { self.0.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.0.size }
    fn get_order_direction(&self) -> Direction { self.0.direction }
    fn extract_body(self) -> OrderInfo { self.0 }
}

impl const Order for LimitOrder {
    fn get_order_id(&self) -> OrderID { self.info.order_id }
    fn get_order_size(&self) -> Size { self.info.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.info.size }
    fn get_order_direction(&self) -> Direction { self.info.direction }
    fn extract_body(self) -> OrderInfo { self.info }
}

impl const PricedOrder for LimitOrder {
    fn get_price(&self) -> Price { self.price }
}