//
//  FPTextureArray.h
//  IronJump
//
//  Created by Filip Kunc on 6/1/10.
//  For license see LICENSE.TXT
//

#import "FPTexture.h"

@interface FPTextureArray : NSObject
{
	NSMutableArray *textures;
}

@property (readonly) NSUInteger count;

- (void)addTexture:(NSString *)fileName;
- (FPTexture *)textureAtIndex:(NSUInteger)index;

@end
