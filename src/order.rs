use crate::types::{OrderDirection, OrderID, OrderSize, Price};

pub(crate) trait Order {
    fn get_order_id(&self) -> OrderID;
    fn get_order_size(&self) -> OrderSize;
    fn mut_order_size(&mut self) -> &mut OrderSize;
    fn get_order_direction(&self) -> OrderDirection;
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
    pub size: OrderSize,
    pub direction: OrderDirection,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MarketOrder(OrderInfo);

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LimitOrder {
    info: OrderInfo,
    price: Price,
}

impl MarketOrder {
    pub fn new(order_id: OrderID, size: OrderSize, direction: OrderDirection) -> MarketOrder {
        MarketOrder(OrderInfo { order_id, size, direction })
    }
}

impl LimitOrder {
    pub fn new(order_id: OrderID, size: OrderSize, direction: OrderDirection, price: Price) -> LimitOrder {
        LimitOrder { info: OrderInfo { order_id, size, direction }, price }
    }
}

impl const Order for MarketOrder {
    fn get_order_id(&self) -> OrderID { self.0.order_id }
    fn get_order_size(&self) -> OrderSize { self.0.size }
    fn mut_order_size(&mut self) -> &mut OrderSize { &mut self.0.size }
    fn get_order_direction(&self) -> OrderDirection { self.0.direction }
    fn extract_body(self) -> OrderInfo { self.0 }
}

impl const Order for LimitOrder {
    fn get_order_id(&self) -> OrderID { self.info.order_id }
    fn get_order_size(&self) -> OrderSize { self.info.size }
    fn mut_order_size(&mut self) -> &mut OrderSize { &mut self.info.size }
    fn get_order_direction(&self) -> OrderDirection { self.info.direction }
    fn extract_body(self) -> OrderInfo { self.info }
}

impl const PricedOrder for LimitOrder {
    fn get_price(&self) -> Price { self.price }
}