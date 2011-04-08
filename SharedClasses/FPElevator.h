//
//  FPElevator.h
//  IronJump
//
//  Created by Filip Kunc on 5/5/10.
//  For license see LICENSE.TXT
//

#import "FPGameProtocols.h"

@interface FPElevator : NSObject <FPGameObject>
{
	float x, y;
	float startX, startY, endX, endY;
	int widthSegments;
	BOOL movingToEnd;
	int animationCounter;
	int textureIndex;
	id<FPGameObject> elevatorEnd;
	NSMutableArray *affectedObjects;
	BOOL isVisible;
}

@property (readwrite, assign) float endX, endY;
	
- (id)initWithWidthSegments:(int)aWidthSegments endX:(float)anEndX endY:(float)anEndY;
- (void)initAffectedObjectsIfNeeded:(id<FPGameProtocol>)game;
- (void)moveCurrentX:(float)offsetX y:(float)offsetY;
- (void)elevatorCollision:(id<FPGameProtocol>)game diffX:(float)diffX diffY:(float)diffY;

@end

@interface FPElevatorEnd : NSObject <FPGameObject>
{
	FPElevator *elevatorStart;
}

- (id)initWithElevatorStart:(FPElevator *)anElevatorStart;

@end

