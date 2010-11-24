//
//  FPPlayer.h
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPGameProtocols.h"

#if TARGET_OS_IPHONE

enum
{
	NSUpArrowFunctionKey        = 0xF700,
	NSDownArrowFunctionKey      = 0xF701,
	NSLeftArrowFunctionKey      = 0xF702,
	NSRightArrowFunctionKey     = 0xF703,
};

#endif

extern const float tolerance;
extern const float maxSpeed;
extern const float upSpeed;
extern const float maxFallSpeed;
extern const float acceleration;
extern const float deceleration;
extern const float changeDirectionSpeed;
extern const int maxSpeedUpCount;

@interface FPPlayer : NSObject <FPGameObject>
{
	float x, y;
	float moveX, moveY;
	float rotation;
	BOOL jumping;

	int speedUpCounter;	
	float alpha;
	BOOL isVisible;
}

@property (readwrite, assign) float moveX, moveY, rotation, alpha;
@property (readwrite, assign) int speedUpCounter;

- (id)initWithWidth:(float)aWidth height:(float)aHeight;
- (BOOL)collisionLeftRight:(id<FPGameProtocol>)game;
- (BOOL)collisionUpDown:(id<FPGameProtocol>)game;
- (void)drawSpeedUp;

@end
